use std::{collections::HashMap, ops::Deref, sync::Arc};

use super::table::{DatabaseTableDefinition, Identifier};

#[derive(Debug)]
pub struct DatabaseDefinition {
    data: Arc<DatabaseDefinitionBuilder>,
}

impl Deref for DatabaseDefinition {
    type Target = DatabaseDefinitionBuilder;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

// impl DatabaseDefinition {
// TODO: Make it so that I can read and parse the existing DB Definition from a file (use serde_json)
//     pub fn from_file(file_path: &str) {
//         todo!()
//     }

//     pub fn to_file(file_path: &str) {
//         todo!()
//     }
//
// TODO: Make it so that I can read and parse the existing DB Definition on a remote DB
//     pub fn from_postgres_database(db_connection_string: &str) {
//         todo!()
//     }
// }

impl DatabaseDefinition {
    /// Creates an empty DatabaseDefinitionData object.
    ///
    /// DatabaseDefinitionData uses the builder pattern to build the object,
    /// and is locked into an [Arc] wrapped in `DatabaseDefinition` with .into()
    /// // TODO: I want to look into locking this without an Arc, although Arc does provide a convenient way of passing the data around without trakcing refs.
    /// // Look into type state pattern
    ///
    /// Example:
    /// ```
    /// let database_definition: DatabaseDefinition
    ///     = DatabaseDefinition::new("new_database")
    ///         .expect("Something went wrong")
    ///         .into();
    /// ```
    #[allow(clippy::new_ret_no_self)]
    pub fn new(name: &str) -> Result<DatabaseDefinitionBuilder, String> {
        DatabaseDefinitionBuilder::new(name)
    }

    /// Creates an empty DatabaseDefinitionData object.  Does not wrap the result in a Result,
    /// instead this method panics if `name` is an invalid identifier.
    /// Example:
    /// ```
    /// let database_definition: DatabaseDefinition
    ///     = DatabaseDefinition::new("new_database").into();
    /// ```
    pub fn new_unchecked(name: &str) -> DatabaseDefinitionBuilder {
        match DatabaseDefinitionBuilder::new(name) {
            Ok(def) => def,
            Err(e) => panic!("DatabaseDefinition::new_unchecked() failed: {}", e),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseDefinitionBuilder {
    pub name: Identifier,
    pub tables: Vec<DatabaseTableDefinition>,
}

impl From<DatabaseDefinitionBuilder> for DatabaseDefinition {
    fn from(mut val: DatabaseDefinitionBuilder) -> Self {
        // This is where we need to preprocess the stuff and make sure the tables are okay / consistent.
        // TODO: ^^^^^ That

        // First, gather all tables into a map:
        // QUESTION: Should this be stored as a map ANYWAY for quick access?
        let map = val
            .tables
            .iter()
            .map(|table| (&table.table_name, table))
            .collect::<HashMap<_, _>>();

        let mut new_tables = Vec::new();
        for table in map.values() {
            for column in table.columns.values() {
                match &column.column_type {
                    super::table::DatabaseColumnType::OneToMany(_child_table_ident) => {
                        // One to Many means / assumes the following things:
                        // 1. Each entry in Parent (current) owns any number of Child objects.
                        // 2. Each Child object has exactly one parent_id
                        // 3. The Child objects are all stored in the other table.
                        // 4. Need to add a FK dependency on the *child* table when processing. This will need to happen in Migrations logic
                        // 5.
                    },
                    super::table::DatabaseColumnType::ManyToMany(child_table_ident) => {
                        // TODO: Add teh FK constraints
                        let join_table = DatabaseTableDefinition::new(&format!(
                            "{}_to_{}",
                            &table.table_name, &child_table_ident
                        ))
                        .expect("Should always be valid")
                        .with_uuid(&format!("{}_id", &table.table_name))
                        .expect("Should always be valid")
                        .with_uuid(&format!("{}_id", &child_table_ident))
                        .expect("Should always be valid");
                        new_tables.push(join_table.into());
                        todo!("Many to Many not yet supported. Need to add constraints still, and impl the non mmigratory parts.");

                        // Many to Many means/assumes the following:
                        // 1. A join table is required.
                        // 2. This join table will have the PARENT_ID / CHILD_ID relationship
                        // 3. At least for the first iter, will be a one-way relationship. Will  think about the best way to model it for two-way
                        // 4. When querying - two joins must be done (join table and table table)

                        // FUTURE FEATURE IDEA: Define ability to use a Metatdata table to specify info about the edge in the join table
                        // Will clustering be needed at all here?
                    },
                    super::table::DatabaseColumnType::OneToOne(_) => {
                        // todo!("One-to-one relationships aren't quite automated yet. Consider using a JSON or String field instead")
                        // One to One means/assumes the following:
                        // 1. This table owns the resulting data.
                        // 2. This table stores a CHILD_ID relationship
                        // 3. When querying, an INNER JOIN must be done
                    },
                    _ => (),
                }
            }
        }
        val.tables.append(&mut new_tables);

        DatabaseDefinition {
            data: Arc::new(val),
        }
    }
}

impl DatabaseDefinitionBuilder {
    pub fn new(name: &str) -> Result<Self, String> {
        Ok(Self {
            name: Identifier::new(name)?,
            tables: Vec::new(),
        })
    }

    pub fn add_table(
        mut self,
        table: DatabaseTableDefinition,
    ) -> Self {
        self.tables.push(table);
        self
    }

    pub fn table<D: Into<DatabaseTableDefinition>>(
        self,
        table: D,
    ) -> Self {
        self.add_table(table.into())
    }
}
