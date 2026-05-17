use std::{
    any::TypeId,
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use sqlx::{Execute, Postgres, QueryBuilder};

use crate::{
    data_definition::table::{self, DatabaseColumnType, TableColumn, TableConstraint},
    data_manager::{GetTableDefinition, PostgresDataProvider},
    migration::Migration,
    queries::Insertable,
    BuildSql,
};

use super::table::{raw_data::TableDefinition, DatabaseTableDefinition, Identifier};

// pub(crate) trait GenericizedTableDefinition: std::any::Any + TableDefinition {}
pub(crate) type TableDef = Arc<DatabaseTableDefinition>;

// impl GenericizedTableDefinition for DatabaseTableDefinition {}

#[derive(Default)]
pub struct DataSystemBuilder {
    resources: HashMap<TypeId, DatabaseTableDefinition>,
    table_name_to_type: HashMap<Identifier, TypeId>,
}

impl DataSystemBuilder {
    pub fn add_resource<T: GetTableDefinition + Send + 'static>(&mut self) {
        // TODO: On get_table_definition, need to include child tables too and add them here.
        // Fixes the issue where I have to add endpoints for child resources even if I don't want them.
        let table_def = T::get_table_definition();
        self.add_table_def::<T>(table_def);
    }
    pub fn with_resource<T: GetTableDefinition + Send + 'static>(mut self) -> Self {
        self.add_resource::<T>();
        self
    }

    pub(crate) fn add_table_def<T: Send + 'static>(
        &mut self,
        table_def: DatabaseTableDefinition,
    ) {
        let type_id = TypeId::of::<T>();
        self.add_table_def_for_type_id(type_id, table_def);
    }

    fn add_table_def_for_type_id(
        &mut self,
        type_id: TypeId,
        table_def: DatabaseTableDefinition,
    ) {
        self.table_name_to_type.insert(table_def.table_name.clone(), type_id);
        for (child_type_id, child_def) in table_def.child_tables() {
            self.add_table_def_for_type_id(child_type_id, *child_def);
        }
        self.resources.insert(type_id, table_def);
    }

    // pub fn get<T: GetTableDefinition + Clone + Send + 'static>(
    //     &self
    // ) -> Option<DatabaseTableDefinition> {
    //     self.resources.get(&TypeId::of::<T>()).map(|t| {
    //         let boxed = <dyn Any>::downcast_ref::<DatabaseTableDefinition>(t).expect(
    //             "Invalid type stored in DataSystem.resources - this should not be possible.
    //             The type exists in the map but failed to downcast.",
    //         );
    //         boxed.clone()

    // }

    pub fn build(self) -> Result<UnconnectedDataSystem, crate::Error> {
        let Self {
            mut resources,
            ..
        } = self;

        let mut stack = resources.clone().into_iter().collect::<Vec<_>>();

        while let Some((_type_id, table_def)) = stack.pop() {
            for (child_type_id, child_def) in table_def.child_tables().into_iter() {
                stack.push((child_type_id, *child_def.clone()));
                resources.insert(child_type_id, *child_def);
            }
        }

        // Put things into a RefCell sot hat we can modify during iteration.
        let mut resources = resources
            .into_iter()
            .map(|(k, v)| (k, RefCell::new(v)))
            .collect::<HashMap<_, _>>();

        // Make sure all child tables are appropriately added.
        // let mut children_of_tables = Vec::new();

        // First pass: Make sure all child tables exist, and modify them where needed.
        let mut new_tables = Vec::new();
        for (_parent_type_id, table_def) in &resources {
            let mut table_def = table_def.borrow_mut();
            let table_name = table_def.table_name();
            let table_id_col =
                table_def.columns().get(&Identifier::new_unchecked("id")).unwrap().clone();
            let columns = table_def.columns().clone();
            // For each column - modify its child table appropriately, if exists.
            for (_col_name, col_def) in columns {
                let col_type = &col_def.column_type;
                type E = DatabaseColumnType;
                match col_type {
                    E::OneToMany(child_name, _table) => {
                        let mut child_table= self.table_name_to_type
                            .get(&child_name)
                            .and_then(|type_id|resources.get(&type_id))
                            .expect(&format!(
                                "Expected child table {child_name} does not exist in Data System. Aborting."
                            )).borrow_mut();
                        // TODO (UUID id requirement): Make this more dynamic when I remove the "must have UUID" requirement
                        let parent_table_col_name = format!("{table_name}_id");
                        let mut new_column = TableColumn::new_uuid(&parent_table_col_name)?
                            .fk_to(table_name.clone(), table_id_col.clone());

                        if !col_def.is_nullable() {
                            new_column = new_column.non_null();
                        }
                        child_table.add_column(new_column);
                    },
                    E::ManyToMany(child_name, _) => {
                        let child_table = self
                            .table_name_to_type
                            .get(&child_name)
                            .and_then(|type_id| resources.get(type_id))
                            .expect(&format!("Expected child table {child_name} to exist"))
                            .borrow();

                        let parent_pk = table_def.get_primary_key().expect("PK should exist");
                        let child_pk = child_table.get_primary_key().expect("PK should exist");

                        let join_table = DatabaseTableDefinition::new(&format!(
                            "{}__to__{}",
                            &table_name, &child_name
                        ))?
                        // TODO: This doesn't *have* to be UUID
                        .with_uuid(&format!("{}_{}", &table_name, &parent_pk.column_name))?
                        .with_uuid(&format!("{}_{}", &child_name, &child_pk.column_name))?;

                        new_tables.push(join_table);
                    },
                    E::OneToOne {
                        col_name: child_name,
                        ..
                    } => {
                        let child_table = self
                            .table_name_to_type
                            .get(&child_name)
                            .and_then(|type_id| resources.get(type_id))
                            .expect(&format!("Expected child table {child_name} to exist"));
                        let child_pk = child_table
                            .borrow()
                            .get_primary_key()
                            .expect("Child table does not have a primary key to associate to.");
                        // Just make sure the child table exists - migrations will handle the actual column name down the line.
                        // TODO: Actually migrations don't do it, it happens in the macro. I should move it here for posterity's sake. Can happen later.
                        let new_col_def = (*col_def)
                            .clone()
                            .fk_to(child_table.borrow().table_name.clone(), child_pk);
                        // Replace the column in the table definition
                        table_def.add_column(new_col_def);
                    },
                    _ => (), // Nothing to preprocess
                }
            }
        }
        // Add the join tables to resources map. Since we won't ever use these directly, we can create some dummy types to fill the map.

        struct DummyTypeId;
        struct DummyTypeId2;
        struct DummyTypeId3;
        struct DummyTypeId4;
        struct DummyTypeId5;
        // macro_rules! create_dummy_type_ids {
        //     ($name:ty,) => {
        //         TypeId::of::<$name>,
        //     };
        //     ($first_name:ty, $(name:ty,)*) => {
        //         TypeId::of::<($first_name, $($name,)*)>,
        //     };
        // }
        // TODO: If I do tuples, this only needs to be one type after all.
        let mut dummy_type_ids = vec![
            TypeId::of::<DummyTypeId>(),
            TypeId::of::<DummyTypeId2>(),
            TypeId::of::<DummyTypeId3>(),
            TypeId::of::<DummyTypeId4>(),
            TypeId::of::<DummyTypeId5>(),
            TypeId::of::<(DummyTypeId, DummyTypeId)>(),
            TypeId::of::<(DummyTypeId, DummyTypeId2)>(),
            TypeId::of::<(DummyTypeId, DummyTypeId3)>(),
            TypeId::of::<(DummyTypeId, DummyTypeId4)>(),
            TypeId::of::<(DummyTypeId, DummyTypeId5)>(),
            TypeId::of::<(DummyTypeId2, DummyTypeId)>(),
            TypeId::of::<(DummyTypeId2, DummyTypeId2)>(),
            TypeId::of::<(DummyTypeId2, DummyTypeId3)>(),
            TypeId::of::<(DummyTypeId2, DummyTypeId4)>(),
            TypeId::of::<(DummyTypeId2, DummyTypeId5)>(),
        ];
        for new_table in new_tables {
            resources.insert(
                dummy_type_ids.pop().expect(
                    "Must modify the code in tailwag_orm to support more than 15 join tables",
                ),
                RefCell::new(new_table),
            );
        }

        // TODO: One last pass, to make sure all the child tables are up to date after the changes.
        // On second thought, I don't think this solves the problem. Need to traverse the whole DB tree bottom up to really make sure.
        fn link_children(
            table_def: &RefCell<DatabaseTableDefinition>,
            resources: &HashMap<TypeId, RefCell<DatabaseTableDefinition>>,
            table_name_to_type: &HashMap<Identifier, TypeId>,
        ) {
            let mut table_def = table_def.borrow_mut();
            for (column_name, child_col) in table_def.columns().clone() {
                match &child_col.column_type {
                    DatabaseColumnType::OneToMany(identifier, _database_table_definition) => {
                        let mut new_col = (*child_col).clone();

                        let child_table = table_name_to_type
                            .get(identifier)
                            .and_then(|i| resources.get(i))
                            .expect("Child table missing");
                        link_children(child_table, resources, table_name_to_type);
                        new_col.column_type = DatabaseColumnType::OneToMany(
                            identifier.clone(),
                            child_table.borrow().clone(),
                        );
                        table_def.columns.insert(column_name, new_col.into());
                    },
                    DatabaseColumnType::ManyToMany(identifier, _database_table_definition) => {
                        let mut new_col = (*child_col).clone();

                        let child_table = table_name_to_type
                            .get(identifier)
                            .and_then(|i| resources.get(i))
                            .expect("Child table missing");
                        link_children(child_table, resources, table_name_to_type);
                        new_col.column_type = DatabaseColumnType::ManyToMany(
                            identifier.clone(),
                            child_table.borrow().clone(),
                        );
                        table_def.columns.insert(column_name, new_col.into());
                    },
                    // DatabaseColumnType::OneToOne(identifier, database_table_definition) => {
                    DatabaseColumnType::OneToOne {
                        col_name: identifier,
                        ref_only,
                        ..
                    } => {
                        let mut new_col = (*child_col).clone();

                        let child_table = table_name_to_type
                            .get(identifier)
                            .and_then(|i| resources.get(i))
                            .expect("Child table missing");
                        link_children(child_table, resources, table_name_to_type);
                        new_col.column_type = DatabaseColumnType::OneToOne {
                            col_name: identifier.clone(),
                            table_def: child_table.borrow().clone(),
                            ref_only: *ref_only,
                        };
                        table_def.columns.insert(column_name, new_col.into());
                    },
                    _ => (),
                }
            }
        }

        for (_parent_type_id, table_def) in &resources {
            link_children(table_def, &resources, &self.table_name_to_type);
        }

        Ok(UnconnectedDataSystem {
            resources: Arc::new(
                resources.into_iter().map(|(k, v)| (k, Arc::new(v.into_inner()))).collect(),
            ),
            table_name_to_type: self.table_name_to_type,
        })
    }
}

#[derive(Clone)]
pub struct UnconnectedDataSystem {
    resources: Arc<HashMap<TypeId, Arc<DatabaseTableDefinition>>>,
    table_name_to_type: HashMap<Identifier, TypeId>,
}
impl UnconnectedDataSystem {
    pub async fn connect(
        &self,
        pool: sqlx::Pool<Postgres>,
    ) -> DataSystem {
        DataSystem {
            resources: self.resources.clone(),
            table_name_to_type: self.table_name_to_type.clone(),
            pool,
        }
    }
}

#[derive(Clone)]
pub struct DataSystem {
    resources: Arc<HashMap<TypeId, Arc<DatabaseTableDefinition>>>,
    table_name_to_type: HashMap<Identifier, TypeId>,
    pool: sqlx::Pool<Postgres>,
}

impl DataSystem {
    pub fn builder() -> DataSystemBuilder {
        DataSystemBuilder::default()
    }

    #[allow(unused)]
    pub(crate) fn get_table_def(
        &self,
        type_id: &TypeId,
    ) -> Option<Arc<DatabaseTableDefinition>> {
        self.resources.get(type_id).cloned()
    }
}

impl DataSystem {
    pub fn get<T: Clone + Insertable + Send + 'static>(&self) -> Option<PostgresDataProvider<T>> {
        self.resources
            .get(&TypeId::of::<T>())
            .map(|t| PostgresDataProvider::new(t.clone(), self.pool.clone()))
    }

    // TODO: Move to internal mod for building
    fn get_prev_tables_if_exists(&self) -> Option<Vec<Arc<DatabaseTableDefinition>>> {
        let tables: Option<Vec<SerializableTableDefinition>> =
            std::fs::read(".table_data/last.migration.json")
                .ok()
                .and_then(|bytes| serde_json::from_slice(bytes.as_slice()).ok());
        tables
            .into_iter()
            .flatten()
            .map(|table| self.to_table_definition(table).map(Arc::new))
            .collect::<Result<_, _>>()
            .ok()
    }

    fn save_prev_tables(
        &self,
        database: Vec<Arc<DatabaseTableDefinition>>,
    ) -> Result<(), std::io::Error> {
        let database = database
            .into_iter()
            .map(|t| SerializableTableDefinition::from((*t).clone()))
            .collect::<Vec<_>>();
        let deser = serde_json::to_string_pretty(&database)?;
        let bytes = deser.as_bytes();

        // Currently panicing - failing to serialize.
        std::fs::create_dir_all(".table_data").ok();
        std::fs::write(".table_data/last.migration.json", bytes)?;
        Ok(())
    }

    pub async fn run_migrations(&self) -> Result<(), crate::Error> {
        let current_config: Vec<Arc<DatabaseTableDefinition>> =
            self.resources.values().map(|table| table.to_owned()).collect();
        if let Some(migrations) =
            Migration::compare(self.get_prev_tables_if_exists(), current_config.clone())
        {
            let mut transaction = self.pool.begin().await?;
            for action in migrations.actions {
                let mut builder = QueryBuilder::new("");
                action.build_sql(&mut builder);
                // builder.build().execute(&mut *transaction).await?;
                let raw_sql = builder.build().sql();
                sqlx::raw_sql(raw_sql).execute(&mut *transaction).await?;
            }
            transaction.commit().await?;
        }
        // TODO: It's crashing when trying to serialze the migrations. Need to dig in.
        self.save_prev_tables(current_config)?;
        Ok(())
    }
}

impl<T> TryFrom<&DataSystem> for PostgresDataProvider<T>
where
    T: Insertable + Clone + Send + 'static,
{
    type Error = String;

    fn try_from(parent: &DataSystem) -> Result<Self, Self::Error> {
        let Some(val) = parent.get::<T>() else {
            return Err("Unable to fetch PostgresDataProvider form DataSystem".to_string());
        };
        Ok(val)
    }
}

/// Intermediary struct for serailzing/deserializing with TypeIds, within the datasystem context.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
struct SerializableTableDefinition {
    pub table_name: Identifier,
    pub columns: BTreeMap<String, TableColumn>,
    #[serde(skip)]
    pub child_tables: HashMap<String, Box<DatabaseTableDefinition>>,
    pub constraints: Vec<TableConstraint>,
}

impl From<DatabaseTableDefinition> for SerializableTableDefinition {
    fn from(value: DatabaseTableDefinition) -> Self {
        Self {
            table_name: value.table_name,
            columns: value.columns.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            child_tables: value
                .child_tables
                .into_iter()
                .map(|(_k, v)| (v.table_name.to_string(), v))
                .collect(),
            constraints: value.constraints,
        }
    }
}

impl DataSystem {
    fn to_table_definition(
        &self,
        def: SerializableTableDefinition,
    ) -> Result<DatabaseTableDefinition, String> {
        let mut child_tables: HashMap<TypeId, Box<DatabaseTableDefinition>> = Default::default();
        let mut not_found_tables: Vec<Identifier> = Default::default();
        for (k, v) in def.child_tables {
            let ident = Identifier::new(k)?;
            match self.table_name_to_type.get(&ident) {
                Some(typeid) => {
                    child_tables.insert(*typeid, v);
                },
                None => {
                    not_found_tables.push(ident);
                },
            }
        }

        // TODO: What was the purpose of this?
        let _not_found_tables = not_found_tables.into_iter().map(
            |i| -> Result<(Identifier, table::TableColumn), String> {
                Ok((i.clone(), TableColumn::uuid(&i)?.into()))
            },
        );
        Ok(DatabaseTableDefinition {
            table_name: def.table_name,
            columns: def
                .columns
                .into_iter()
                .map(|(k, v)| match Identifier::new(k) {
                    Ok(i) => Ok((i, v)),
                    Err(e) => Err(e),
                })
                // .chain(not_found_tables)
                .collect::<Result<_, _>>()?,
            child_tables,
            constraints: def.constraints,
        })
    }
}
