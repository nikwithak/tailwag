use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::queries::Insertable;
use crate::BuildSql;
use crate::{data_definition::table::DatabaseTableDefinition, data_manager::GetTableDefinition};

use crate::data_definition::table::Identifier;

use super::{TableColumnConstraint, TableColumnConstraintDetail};

#[allow(unused)]
trait ForeignKeyObject
where
    Self: GetTableDefinition + Insertable + Clone,
{
}

pub(crate) type ObjectRepr = HashMap<Identifier, ColumnValue>;

#[derive(Clone, Debug)]
pub enum ColumnValue {
    Boolean(bool),                    // BOOL or BOOLEAN
    Int(i64),                         // INT
    Float(f64),                       // FLOAT
    String(String),                   // VARCHAR or TEXT
    Timestamp(chrono::NaiveDateTime), // TIMESTAMP
    Uuid(uuid::Uuid),                 // UUID
    Json(String),                     // JSONB
    OneToMany {
        child_table: Identifier,
        values: Vec<Box<ObjectRepr>>,
    },
    OneToOne {
        child_table: Identifier,
        value: Box<ObjectRepr>,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub enum DatabaseColumnType {
    Boolean,   // BOOL or BOOLEAN
    Int,       // INT
    Float,     // FLOAT
    String,    // VARCHAR or TEXT
    Timestamp, // TIMESTAMP
    Uuid,      // UUID
    Json,      // JSONB

    // These next few that define relationship types are a hacky way of building cross-table relationships -
    // I'm dealing with a consequence of deciding that tables would be locked once they were fully built, but this will require that I re-do them down the line.
    OneToMany(Identifier, DatabaseTableDefinition), // TODO: Impl this all the way through
    // ManyToOne(DatabaseTableDefinition),
    ManyToMany(Identifier, DatabaseTableDefinition), // TODO: Will need to figure out how I want to represent JoinTables here
    OneToOne {
        col_name: Identifier,
        table_def: DatabaseTableDefinition,
        ref_only: bool,
    }, // TODO: With [inline] macro attribute, can make this inline JSON when needed. Depeneds on if we want sortability / searchability or not
}

impl DatabaseColumnType {
    pub fn as_str(&self) -> &str {
        match self {
            DatabaseColumnType::Boolean => "BOOL",
            DatabaseColumnType::Int => "INT",
            DatabaseColumnType::Float => "FLOAT",
            DatabaseColumnType::String => "VARCHAR",
            DatabaseColumnType::Timestamp => "TIMESTAMP",
            DatabaseColumnType::Uuid => "UUID",
            DatabaseColumnType::Json => "JSONB",
            DatabaseColumnType::OneToMany(_, _) => "UUID",
            DatabaseColumnType::ManyToMany {
                ..
            } => todo!(),
            DatabaseColumnType::OneToOne {
                ..
            } => "UUID", // TODO: These types of relationships only work with id: uuid. This is a big debt.
        }
    }
}

/// Represents a table column. Primarily-based on the PostgreSQL spec for table definition
/// The goal is to get full parity with PostgreSQL.
/// [ref](https://www.postgresql.org/docs/current/sql-createtable.html)
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct TableColumn {
    data: Arc<TableColumnData>,
}

impl Deref for TableColumn {
    type Target = TableColumnData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl From<TableColumnData> for Identifier {
    fn from(val: TableColumnData) -> Self {
        val.column_name.clone()
    }
}
impl From<TableColumn> for Identifier {
    fn from(val: TableColumn) -> Self {
        val.data.column_name.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct TableColumnData {
    // TODO: Is parent_table_name needed here??? Parent should track it, doesn't  need a call back (why did I add this in the first place??)
    // pub parent_table_name: Identifier, // Identifier is just an Arc<String> (with validation) under the hood
    pub column_name: Identifier,         // Part of spec
    pub column_type: DatabaseColumnType, // Part of spec - named `data_type` in spec
    // pub compression_method: Optional<CompressionMethod>, // TODO: Adds `COMPRESSION compression_method` after `data_type`
    // pub collation: Optional<Collation>, // TODO: Adds `COMPRESSION compression_method` after `data_type`
    pub constraints: Vec<TableColumnConstraint>,
}

impl From<TableColumnData> for TableColumn {
    fn from(val: TableColumnData) -> Self {
        TableColumn {
            data: Arc::new(val),
        }
    }
}

impl TableColumnData {
    pub fn is_nullable(&self) -> bool {
        self.constraints
            .iter()
            .all(|constraint| TableColumnConstraintDetail::NotNull != *constraint.detail)
    }
    pub fn is_pk(&self) -> bool {
        self.constraints.iter().any(|constraint| {
            matches!(&*constraint.detail, TableColumnConstraintDetail::PrimaryKey(_))
        })
    }
}

impl TableColumnData {
    pub fn non_null(mut self) -> Self {
        self.constraints.push(TableColumnConstraint::non_null());
        self
    }

    pub fn primary_key(mut self) -> Self {
        self.constraints.push(TableColumnConstraint::primary_key());
        self
    }
    pub fn pk(self) -> Self {
        self.primary_key()
    }

    pub fn fk_to(
        self,
        ref_table: Identifier,
        ref_column: TableColumn,
    ) -> Self {
        self.foreign_key_to(ref_table, ref_column)
    }

    pub fn foreign_key_to(
        mut self,
        ref_table: Identifier,
        ref_column: TableColumn,
    ) -> Self {
        self.constraints.push(TableColumnConstraint::foreign_key(ref_table, ref_column));
        self
    }
}

impl TableColumn {
    /// Creates a new TableColumn.
    ///
    /// Before using TableColumn::new, see if any of the provided helper
    /// `TableColumn::new_*` functions fit your use case.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        column_name: &str,
        column_type: DatabaseColumnType,
        constraints: Vec<TableColumnConstraint>,
    ) -> Result<TableColumnData, String> {
        Ok(TableColumnData {
            // parent_table_name,
            column_name: Identifier::new(column_name)?,
            column_type,
            constraints,
        })
    }

    pub fn int(column_name: &str) -> Result<TableColumnData, String> {
        Self::new_int(column_name)
    }
    pub fn timestamp(column_name: &str) -> Result<TableColumnData, String> {
        Self::new_timestamp(column_name)
    }
    pub fn string(column_name: &str) -> Result<TableColumnData, String> {
        Self::new_string(column_name)
    }
    pub fn float(column_name: &str) -> Result<TableColumnData, String> {
        Self::new_float(column_name)
    }
    pub fn uuid(column_name: &str) -> Result<TableColumnData, String> {
        Self::new_uuid(column_name)
    }
    pub fn bool(column_name: &str) -> Result<TableColumnData, String> {
        Self::new_bool(column_name)
    }

    pub fn new_int(column_name: &str) -> Result<TableColumnData, String> {
        Ok(TableColumnData {
            column_name: Identifier::new(column_name)?,
            column_type: DatabaseColumnType::Int,
            constraints: vec![],
        })
    }

    pub fn new_string(column_name: &str) -> Result<TableColumnData, String> {
        Ok(TableColumnData {
            column_name: Identifier::new(column_name)?,
            column_type: DatabaseColumnType::String,
            constraints: vec![],
        })
    }

    pub fn new_timestamp(column_name: &str) -> Result<TableColumnData, String> {
        Ok(TableColumnData {
            column_name: Identifier::new(column_name)?,
            column_type: DatabaseColumnType::Timestamp,
            constraints: vec![],
        })
    }

    pub fn new_float(column_name: &str) -> Result<TableColumnData, String> {
        Ok(TableColumnData {
            column_name: Identifier::new(column_name)?,
            column_type: DatabaseColumnType::Float,
            constraints: vec![],
        })
    }

    pub fn new_uuid(column_name: &str) -> Result<TableColumnData, String> {
        Ok(TableColumnData {
            column_name: Identifier::new(column_name)?,
            column_type: DatabaseColumnType::Uuid,
            constraints: vec![],
        })
    }

    pub fn new_bool(column_name: &str) -> Result<TableColumnData, String> {
        Ok(TableColumnData {
            column_name: Identifier::new(column_name)?,
            column_type: DatabaseColumnType::Boolean,
            constraints: vec![],
        })
    }
}

impl BuildSql for TableColumn {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        // sql.push_bind(self.column_name.to_string());
        sql.push(self.column_name.to_string()).push(" ").push(self.column_type.as_str());

        let constraints_iter = self.constraints.iter();
        for constraint in constraints_iter {
            constraint.build_sql(sql);
        }
    }
}
