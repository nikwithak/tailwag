use crate::{
    data_definition::table::{DatabaseColumnType, Identifier, TableColumn, TableConstraint},
    BuildSql,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AlterTableAction {
    Rename(Identifier),
    AddColumn(TableColumn),   // TODO
    DropColumn(Identifier),   // TODO
    AlterColumn(AlterColumn), // TODO
    // TODO: Add the rest of the actions.
    // Ref: https://www.postgresql.org/docs/current/sql-altertable.html
    AddConstraint(TableConstraint),
    AlterConstraint(),
    DropConstraint(TableConstraint),
}

impl BuildSql for AlterTableAction {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        type E = AlterTableAction;

        match self {
            E::Rename(ident) => {
                sql.push("RENAME TO ").push(ident.to_string());
            },
            E::AddColumn(table_column) => {
                sql.push("ADD COLUMN IF NOT EXISTS ");
                table_column.build_sql(sql);
            },
            E::DropColumn(ident) => {
                sql.push("DROP COLUMN IF EXISTS ").push(ident.to_string());
            },
            E::AlterColumn(alter_column) => alter_column.build_sql(sql),
            E::AddConstraint(constraint) => {
                sql.push("ADD ");
                constraint.build_sql(sql)
            },
            E::AlterConstraint() => todo!(),
            E::DropConstraint(constraint) => {
                sql.push("DROP CONSTRAINT IF EXISTS ").push(
                    constraint
                        .name
                        .as_ref()
                        .map(|c| c.to_string())
                        .unwrap_or("UNNAMED_CONSTRAINT".to_string()),
                );
            },
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AlterTable {
    pub table_name: Identifier,
    pub actions: Vec<AlterTableAction>,
}

impl BuildSql for AlterTable {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        let actions = self.actions.iter().peekable();
        for action in actions {
            sql.push("ALTER TABLE IF EXISTS ").push(self.table_name.to_string()).push(" ");
            action.build_sql(sql);
            sql.push(";\n");
        }
    }
}

// TODO: Part of _SetStorage() requirement
// enum StorageType {
//     Plain,
//     External,
//     Extended,
//     Main,
// }

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AlterColumnAction {
    SetType(DatabaseColumnType), // TODO: Look at the other options here
    SetNullability(bool),        // True if nullable, False if not.
                                 // _DropExpression,            // TODO: Unsupported yet
                                 // _AddGenerated(),            // TODO: Unsupported yet
                                 // _SetGenerated(),            // TODO: Unsupported yet
                                 // _DropIdentity,              // TODO: Unsupported yet Always use IF EXISTS
                                 // _SetStatistics(i64),        // TODO: Unsupported yet
                                 // _SetAttribute(),            // TODO: Unsupported yet
                                 // _Reset(),                   // TODO: Unsupported yet
                                 // _SetStorage(StorageType),   // TODO: Unsupported yet
                                 // _SetCompression(_CompressionMethod), // TODO: Unsupported yet
}

impl BuildSql for AlterColumnAction {
    fn build_sql(
        &self,

        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        match self {
            AlterColumnAction::SetType(t) => sql.push("TYPE ").push(t.as_str()),
            AlterColumnAction::SetNullability(nullable) => {
                if *nullable {
                    sql.push("DROP NOT NULL")
                } else {
                    sql.push("SET NOT NULL")
                }
            },
        };
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AlterColumn {
    pub column_name: Identifier,
    pub actions: Vec<AlterColumnAction>,
}

impl BuildSql for AlterColumn {
    fn build_sql(
        &self,
        sql: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        let mut actions = self.actions.iter().peekable();
        while let Some(action) = actions.next() {
            sql.push("ALTER COLUMN ").push(self.column_name.to_string()).push(" ");
            action.build_sql(sql);
            if actions.peek().is_some() {
                sql.push(", ");
            }
        }
    }
}
