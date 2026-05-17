use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use sqlx::{Pool, Postgres};

use crate::{
    data_definition::{
        data_system::TableDef,
        table::{
            raw_data::TableDefinition, ForeignKeyConstraint, Identifier, TableColumn,
            TableConstraint, TableConstraintDetail,
        },
    },
    migration::{AlterColumn, AlterColumnAction, AlterTableAction},
    BuildSql,
};

use super::{AlterTable, CreateTable};

#[derive(Clone, Debug)]
pub enum MigrationAction {
    AlterTable(AlterTable),
    CreateTable(CreateTable),
    DropTable(Identifier),
}

impl MigrationAction {
    pub fn get_table_name(&self) -> &Identifier {
        match self {
            MigrationAction::AlterTable(alter_table) => &alter_table.table_name,
            MigrationAction::CreateTable(create_table) => &create_table.table_definition.table_name,
            MigrationAction::DropTable(identifier) => identifier,
        }
    }
}

impl BuildSql for MigrationAction {
    fn build_sql(
        &self,
        builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    ) {
        match self {
            MigrationAction::AlterTable(alter_table) => alter_table.build_sql(builder),
            MigrationAction::CreateTable(create_table) => create_table.build_sql(builder),
            MigrationAction::DropTable(table_ident) => {
                builder.push("DROP TABLE IF EXISTS ").push(&**table_ident);
            },
        };
    }
}

#[derive(Clone)]
pub struct Migration {
    pub actions: Vec<MigrationAction>,
}

impl Migration {
    pub async fn run(
        self,
        db_pool: &Pool<Postgres>,
    ) -> Result<(), crate::Error> {
        let mut builder = sqlx::QueryBuilder::new("");
        self.build_sql(&mut builder);
        builder.build().execute(db_pool).await?;

        Ok(())
    }
}

impl BuildSql for Migration {
    fn build_sql(
        &self,
        builder: &mut sqlx::QueryBuilder<'_, Postgres>,
    ) {
        for alter_table in &self.actions {
            alter_table.build_sql(builder);
            builder.push("\n");
        }
    }
}

impl Migration {
    pub(crate) fn compare(
        before: Option<Vec<TableDef>>,
        after: Vec<TableDef>,
    ) -> Option<Self> {
        let mut actions: Vec<MigrationAction> = Vec::new();

        fn build_table_map(db_def: Vec<TableDef>) -> HashMap<Identifier, TableDef> {
            let map = db_def.iter().fold(HashMap::new(), |mut acc, table| {
                acc.insert(table.table_name(), table.clone());
                acc
            });
            map
        }

        if let Some(before) = before {
            // Build a map for quick lookup of after_tables, then compare each
            let mut after_tables = build_table_map(after.clone());
            for table_before in before {
                match after_tables
                    .remove(&table_before.table_name())
                    .map(|after_table| Self::compare_tables(table_before.clone(), after_table))
                {
                    Some(None) => {
                        println!("NO CHANGES");
                        // No changes to the table
                        // Do nothing
                    },
                    Some(Some(mut table_diff)) => {
                        println!("[MODIFY TABLE] {}", table_before.table_name());
                        // println!("before: {:?}", table_before);
                        // Table has been modified
                        actions.append(&mut table_diff.actions);
                    },
                    None => {
                        println!("[DELETE TABLE] {}", table_before.table_name());
                        // Table was not found in new tables, meaning it was deleted
                        // TODO: At the end, compare any deleted/added tables to see if there was just some renaming done.
                        let action = MigrationAction::DropTable(table_before.table_name().clone());
                        actions.push(action);
                    },
                };
            }

            // Any remaining tables are new - add the CreateTable actions
            actions.append(
                &mut after_tables
                    .values()
                    .map(|table| MigrationAction::CreateTable(CreateTable::new(table.clone()))) //(*table).clone())))
                    .collect(),
            )
        } else {
            // New database - only creates!
            let mut create_table_actions: Vec<MigrationAction> = after
                .iter()
                .map(|t| MigrationAction::CreateTable(CreateTable::new(t.clone())))
                .collect();
            actions.append(&mut create_table_actions);
        }

        // Sort actions by dependency order - children must be processed before their parents.
        let tables = build_table_map(after);

        // Use Kahn's Algorithm for a topological sort.
        let actions = {
            let node_map: HashMap<Identifier, MigrationAction> =
                actions.into_iter().map(|a| (a.get_table_name().clone(), a)).collect();
            let known_ids: HashSet<Identifier> = node_map.keys().cloned().collect();

            // in_degree lid] = number of constraints not yet satisfied.
            let mut in_degree: HashMap<Identifier, usize> =
                known_ids.iter().map(|i| (i.clone(), 0)).collect();

            let mut edges: HashMap<Identifier, Vec<Identifier>> =
                known_ids.iter().map(|id| (id.clone(), vec![])).collect();

            for parent in node_map.values() {
                if let MigrationAction::DropTable(_) = parent {
                    // Skip the deletes. This may cause some order of operations issues in the future, but it fixes the immediate use case
                    continue;
                }
                let table = tables.get(&parent.get_table_name()).ok_or(()).unwrap(); // TODO: Gracefully error
                for child in table.columns().values() {
                    match &child.column_type {
                        crate::data_definition::table::DatabaseColumnType::OneToMany(
                            _col_nameidentifiei,
                            table_def,
                        ) => {
                            // Child has references to parent, so parent must come first.
                            if known_ids.contains(&table_def.table_name()) {
                                edges
                                    .entry(parent.get_table_name().clone())
                                    .or_default()
                                    .push(table_def.table_name());
                                *in_degree.entry(table_def.table_name()).or_insert(0) += 1;
                            }
                        },
                        crate::data_definition::table::DatabaseColumnType::OneToOne {
                            table_def,
                            ..
                        } => {
                            // Parent has refs to child, so child must come first.
                            if known_ids.contains(&table_def.table_name()) {
                                edges
                                    .entry(table_def.table_name())
                                    .or_default()
                                    .push(parent.get_table_name().clone());
                                *in_degree.entry(parent.get_table_name().clone()).or_insert(0) += 1;
                            }
                        },
                        crate::data_definition::table::DatabaseColumnType::ManyToMany(..) => {
                            todo!("Many to Many relationships are not yet supported.");
                        },
                        _ => (),
                    }
                }
            }

            // Kahn's algo: Start with all nodes that have no predecessors.
            let mut queue: VecDeque<Identifier> = VecDeque::new();
            let mut first_batch: Vec<_> = in_degree
                .iter()
                .filter(|(_, &deg)| deg == 0)
                .map(|(id, _)| id.clone())
                .collect();
            // We sort the values for deterministic / consistent order. Not mandatory, but makes testing consisent.
            first_batch.sort_unstable();
            queue.extend(first_batch);

            let mut result: Vec<MigrationAction> = Vec::with_capacity(node_map.len());
            while let Some(id) = queue.pop_front() {
                result.push(node_map[&id].clone());
                if let Some(neighbors) = edges.get(&id) {
                    let mut next_batch: Vec<Identifier> = Vec::new();
                    for neighbor in neighbors {
                        let deg = in_degree.entry(neighbor.clone()).or_insert(0);
                        *deg -= 1;
                        if *deg == 0 {
                            next_batch.push(neighbor.clone());
                        }
                    }
                    next_batch.sort_unstable();
                    queue.extend(next_batch);
                }
            }

            result
        };

        if !actions.is_empty() {
            Some(Self {
                actions,
            })
        } else {
            None
        }
    }

    /// Returns `Some<Migration>` representing the steps required to go from `before` to `after`, or None if the inputs are the same.
    ///
    /// # Arguments
    ///
    /// * `before` - The existing table definition, before the new changes are applied.
    /// * `after` - The new table definition, currently in use.
    ///
    /// # Returns
    ///
    fn compare_tables(
        before: TableDef,
        after: TableDef,
    ) -> Option<Self> {
        let mut actions = Vec::<AlterTableAction>::new();

        // Name changed
        if before.table_name() != after.table_name() {
            actions.push(AlterTableAction::Rename(after.table_name()));
        }

        // Build a map for quick lookup of after_tables, then compare each
        let mut after_columns: HashMap<&Identifier, &TableColumn> =
            after.columns().iter().collect();
        for old_column in before.columns().values() {
            match old_column.column_type {
                crate::data_definition::table::DatabaseColumnType::ManyToMany(..)
                | crate::data_definition::table::DatabaseColumnType::OneToMany(..) => {
                    // These are handled differently - in preprocessing the DataSystem creates the actual table columns / join tables, so those should be handled
                    // separately. The other tables should already be present in the DatabaseTableDefinition,
                    continue;
                },
                _ => (),
            };
            match after_columns.remove(&old_column.column_name) {
                Some(new_column) => {
                    let mut alter_column_actions = Vec::new();
                    if !old_column.column_type.eq(&new_column.column_type) {
                        alter_column_actions
                            .push(AlterColumnAction::SetType(new_column.column_type.clone()));
                    }

                    // * NONNULL calculation - Compares `NotNull`
                    {
                        // TODO: [TECH DEBT] Uggggh this is really hacky. Wanna clean this up later.
                        // Find the existence of a `NotNull` constraint. If it does *not* exist (`.is_none()`) then the field *is* nullable.
                        // A confusing mess of double negative magic going on here.
                        let old_is_nullable = !old_column.constraints.iter().any(|c| match *c.detail {
                                // Allowed null unless NOT NULL
                                crate::data_definition::table::TableColumnConstraintDetail::NotNull => true,
                                _ => false,
                            });
                        let new_is_nullable = !new_column.constraints.iter().any(|c| {
                            matches!(
                                *c.detail,
                                crate::data_definition::table::TableColumnConstraintDetail::NotNull
                            )
                        });
                        if old_is_nullable != new_is_nullable {
                            alter_column_actions
                                .push(AlterColumnAction::SetNullability(new_is_nullable));
                        }
                    }

                    // * TODO: Foreign Key Changes
                    // We compare the columns, and add the _TABLE'S_ FK constraint, because we can't add the constraint to column except at creation.
                    // TODO: Do another pass for the FK constraints.
                    {
                        let old_fk = old_column.constraints.iter().find_map(|c| match &*c.detail {
                            crate::data_definition::table::TableColumnConstraintDetail::References(fk) => Some(fk),
                            _ => None,
                        });
                        let new_fk = new_column.constraints.iter().find_map(|c| match &*c.detail {
                            crate::data_definition::table::TableColumnConstraintDetail::References(fk) => Some(fk),
                            _ => None,
                        });
                        match (old_fk, new_fk) {
                            (None, None) => None,
                            (None, Some(new_fk)) => {
                                // FK was added
                                Some(AlterTableAction::AddConstraint(TableConstraint {
                                    name: None, // TODO: Infer name. I might need to rework how these constraints work :facepalm:
                                    detail: Arc::new(TableConstraintDetail::ForeignKey(
                                        ForeignKeyConstraint {
                                            ref_table: new_fk.ref_table.clone(),
                                            ref_columns: vec![new_fk
                                                .ref_column
                                                .as_ref()
                                                .unwrap()
                                                .clone()],
                                            columns: vec![new_column.clone()],
                                            match_type: new_fk.match_type.clone(),
                                            on_delete_action: new_fk.on_delete_action.clone(),
                                            on_update_action: new_fk.on_update_action.clone(),
                                        },
                                    )),
                                }))
                            },
                            (Some(_), None) => todo!(), // DELETE constraint from column. Need a consistent way to name constraints in order to do this.
                            (Some(old_fk), Some(new_fk)) => {
                                if old_fk.ne(new_fk) {
                                    todo!("Support for updating FKs not yet implemented")
                                } else {
                                    None
                                }
                            }, // COMPARE constraints
                        };
                    }

                    if !alter_column_actions.is_empty() {
                        actions.push(AlterTableAction::AlterColumn(AlterColumn {
                            column_name: new_column.column_name.clone(),
                            actions: alter_column_actions,
                        }));
                    }
                },
                None => {
                    // Column Deleted
                    actions.push(AlterTableAction::DropColumn(old_column.column_name.clone()));
                },
            }
        }

        // Any remaining columns are new
        for column in after_columns.values() {
            // Special handling for Many-to-Many or One-to-Many relations
            match column.column_type {
                crate::data_definition::table::DatabaseColumnType::ManyToMany(..)
                | crate::data_definition::table::DatabaseColumnType::OneToMany(..) => {
                    // These are handled differently - in preprocessing the DataSystem creates the actual table columns / join tables, so those should be handled
                    // separately. The other tables should already be present in the DatabaseTableDefinition,
                    continue;
                },
                _ => (),
            };
            actions.push(AlterTableAction::AddColumn((*column).clone()));
        }

        if !actions.is_empty() {
            Some(Self {
                actions: vec![MigrationAction::AlterTable(AlterTable {
                    table_name: after.table_name().clone(),
                    actions,
                })],
            })
        } else {
            None
        }
    }
}
