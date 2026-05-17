use std::{ops::Deref, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    data_definition::table::{Identifier, TableColumn},
    BuildSql,
};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default)]
pub struct PrimaryKeyColumnConstraint {
    index_parameters: Option<IndexParameters>,
}

impl BuildSql for PrimaryKeyColumnConstraint {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        sql.push("PRIMARY KEY ");
        if let Some(params) = &self.index_parameters {
            params.build_sql(sql);
        }
    }
}

#[allow(unused)]
struct CheckExpressionConstraint {}

impl BuildSql for CheckExpressionConstraint {
    fn build_sql(
        &self,
        _sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        todo!()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default)]
pub struct IndexParameters {}

impl BuildSql for IndexParameters {
    fn build_sql(
        &self,
        _sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        todo!()
    }
}

impl BuildSql for Vec<Identifier> {
    fn build_sql(
        &self,
        builder: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        let mut idents = self.iter().peekable();
        while let Some(col) = idents.next() {
            builder.push_bind(col.to_string());
            if idents.peek().is_some() {
                builder.push(", ");
            }
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Default)]
pub struct UniqueColumnConstraint {
    is_null_distinct: bool,
    index_parameters: Option<IndexParameters>,
}

impl BuildSql for UniqueColumnConstraint {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        sql.push("UNIQUE");
        if self.is_null_distinct {
            sql.push(" NULLS DISTINCT");
        }
        // TODO: index_parameters impl
    }
}

/// Foreign Key
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct ReferencesConstraint {
    pub ref_table: Identifier,
    pub ref_column: Option<Identifier>,
    pub match_type: Option<ReferencesConstraintMatchType>,
    pub on_delete_action: Option<ReferentialAction>,
    pub on_update_action: Option<ReferentialAction>,
}

impl ReferencesConstraint {
    fn new(
        ref_table: Identifier,
        ref_column: Identifier,
    ) -> Self {
        Self {
            ref_table,
            ref_column: Some(ref_column),
            match_type: None,
            on_delete_action: None,
            on_update_action: None,
        }
    }

    fn _compare(
        _old: Self,
        _new: Self,
    ) -> Option<String> {
        todo!()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum ReferentialAction {
    NoAction,
    Restrict,
    Cascade,
    SetNull(Vec<Identifier>), // ColumnName - TODO: Find a way to define / enforce this. At least one is required.
    SetDefault(Vec<Identifier>),
}

impl BuildSql for ReferentialAction {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        match self {
            ReferentialAction::NoAction => sql.push("NO ACTION"),
            ReferentialAction::Restrict => sql.push("RESTRICT"),
            ReferentialAction::Cascade => sql.push("CASCADE"),
            ReferentialAction::SetNull(column_names) => {
                sql.push("SET NULL ( ");
                column_names.build_sql(sql);
                sql.push(" )")
            },
            ReferentialAction::SetDefault(column_names) => {
                sql.push(" SET DEFAULT (");
                column_names.build_sql(sql);
                sql.push(" )")
            },
        };
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum ReferencesConstraintMatchType {
    Full,
    Partial,
    Simple,
}

impl BuildSql for ReferencesConstraintMatchType {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        sql.push(match self {
            ReferencesConstraintMatchType::Full => "MATCH FULL",
            ReferencesConstraintMatchType::Partial => "MATCH PARTIAL",
            ReferencesConstraintMatchType::Simple => "MATCH SIMPLE",
        });
    }
}

// TODO: Aside from implemetning this, I need to refactor how I handle (some?) constraints that reference other tables, to avoid referencing the table before it exists.

// sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
impl BuildSql for ReferencesConstraint {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        sql.push("REFERENCES ");
        sql.push(self.ref_table.to_string());
        sql.push(' ');
        if let Some(column) = &self.ref_column {
            sql.push('(');
            sql.push(column);
            sql.push(')');
        }
        if let Some(match_type) = &self.match_type {
            match_type.build_sql(sql);
        }
        if let Some(ref_action) = &self.on_delete_action {
            sql.push(" ON DELETE ");
            ref_action.build_sql(sql);
        }
        if let Some(ref_action) = &self.on_delete_action {
            sql.push(" ON UPDATE ");
            ref_action.build_sql(sql);
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct TableColumnConstraint {
    pub name: Option<Identifier>,
    pub detail: Arc<TableColumnConstraintDetail>,
}

impl Deref for TableColumnConstraint {
    type Target = TableColumnConstraintDetail;
    fn deref(&self) -> &Self::Target {
        &self.detail
    }
}

impl TableColumnConstraint {
    pub fn new(constraint: TableColumnConstraintDetail) -> Self {
        Self {
            name: None,
            detail: Arc::new(constraint),
        }
    }

    /// Builds a default Unique constraint.
    pub fn unique() -> Self {
        Self {
            name: None,
            detail: Arc::new(
                TableColumnConstraintDetail::Unique(UniqueColumnConstraint::default()),
            ),
        }
    }

    /// Builds a basic foreign key restraint for a column
    pub fn foreign_key(
        ref_table_name: Identifier,
        ref_column: TableColumn,
    ) -> Self {
        Self {
            name: None,
            detail: Arc::new(TableColumnConstraintDetail::References(ReferencesConstraint::new(
                ref_table_name.clone(),
                ref_column.column_name.clone(),
            ))),
        }
    }

    pub fn primary_key() -> Self {
        Self {
            name: None,
            detail: Arc::new(TableColumnConstraintDetail::PrimaryKey(
                PrimaryKeyColumnConstraint::default(),
            )),
        }
    }

    pub fn non_null() -> Self {
        Self {
            name: None,
            detail: Arc::new(TableColumnConstraintDetail::NotNull),
        }
    }
}

impl BuildSql for TableColumnConstraint {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        sql.push(" ");
        if let Some(name) = &self.name {
            sql.push("CONSTRAINT ");
            sql.push_bind(name.to_string());
            sql.push(" ");
        }
        self.detail.build_sql(sql)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum TableColumnConstraintDetail {
    NotNull,
    Null,
    // Check(CheckExpressionConstraint), // TODO
    // Default(_), // TODO
    // Generated(GeneratedConstraint), // TODO
    Unique(UniqueColumnConstraint),
    PrimaryKey(PrimaryKeyColumnConstraint),
    References(ReferencesConstraint),
}

impl BuildSql for TableColumnConstraintDetail {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        match self {
            TableColumnConstraintDetail::NotNull => {
                sql.push("NOT NULL".to_owned());
            },
            TableColumnConstraintDetail::Null => {
                sql.push("NULL".to_owned());
            },
            // TableColumnConstraint::Check(check) => check.as_sql(),
            // TableColumnConstraint::Default(_) => todo!(),
            // TableColumnConstraint::Generated(_) => todo!(),
            TableColumnConstraintDetail::Unique(unique) => unique.build_sql(sql),
            TableColumnConstraintDetail::PrimaryKey(pk) => pk.build_sql(sql),
            TableColumnConstraintDetail::References(fk) => fk.build_sql(sql),
        }
    }
}
