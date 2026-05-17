use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::BuildSql;

use super::{
    Identifier, IndexParameters, ReferencesConstraintMatchType, ReferentialAction, TableColumn,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct TableConstraint {
    pub name: Option<Identifier>,
    pub detail: Arc<TableConstraintDetail>,
}

impl BuildSql for TableConstraint {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        if let Some(name) = &self.name {
            sql.push("CONSTRAINT ").push_bind(name.to_string()).push(" ");
        }
        self.detail.build_sql(sql);
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum TableConstraintDetail {
    Unique(UniqueConstraint),
    PrimaryKey(PrimaryKeyConstraint),
    ForeignKey(ForeignKeyConstraint),
}

impl BuildSql for TableConstraintDetail {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        match self {
            Self::ForeignKey(fk) => fk.build_sql(sql),
            Self::Unique(un) => un.build_sql(sql),
            Self::PrimaryKey(pk) => pk.build_sql(sql),
        };
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct UniqueConstraint {
    is_null_distinct: bool,
    index_parameters: Option<IndexParameters>,
    columns: Vec<TableColumn>,
}

impl BuildSql for UniqueConstraint {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        sql.push("UNIQUE ");
        if self.is_null_distinct {
            sql.push("NULLS DISTINCT ");
        }
        let columns = self.columns.iter().map(|col| col.column_name.clone()).collect::<Vec<_>>();
        columns.build_sql(sql);
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct PrimaryKeyConstraint {
    index_parameters: Option<IndexParameters>,
    pub columns: Vec<TableColumn>,
}

impl BuildSql for PrimaryKeyConstraint {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        sql.push("PRIMARY KEY ");
        let columns = self.columns.iter().map(|col| col.column_name.clone()).collect::<Vec<_>>();
        columns.build_sql(sql);

        // TODO: IndexParameters impl
    }
}

impl BuildSql for ForeignKeyConstraint {
    fn build_sql(
        &self,
        builder: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        let Self {
            ref_table,
            ref_columns,
            columns,
            ..
        } = self;
        builder.push("FOREIGN_KEY(");
        // TODO: Loop through columns
        columns
            .iter()
            .map(|col| col.column_name.clone())
            .collect::<Vec<_>>()
            .build_sql(builder);
        builder.push(") ").push("REFERENCES ").push(&**ref_table).push("(");
        ref_columns.build_sql(builder);
        builder.push(")");
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct ForeignKeyConstraint {
    pub ref_table: Identifier,
    pub ref_columns: Vec<Identifier>, // TODO: Yank this to a more definitive mapping from columns and ref_columns
    pub columns: Vec<TableColumn>,
    pub match_type: Option<ReferencesConstraintMatchType>,
    pub on_delete_action: Option<ReferentialAction>,
    pub on_update_action: Option<ReferentialAction>,
}
