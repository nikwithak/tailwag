use serde::{Deserialize, Serialize};
use std::{fmt::Display, marker::PhantomData, sync::Arc};

use crate::{
    data_definition::table::{DatabaseTableDefinition, Identifier, JsonBuildObjectResponse},
    object_management::{
        delete::DeleteStatement, insert::InsertStatement, update::UpdateStatement,
    },
    BuildSql,
};

use super::Filter;

pub struct Query<T> {
    pub(crate) table: Arc<DatabaseTableDefinition>,
    pub(crate) filter: Option<Filter>,
    pub(crate) limit: Option<usize>,
    pub(crate) _t: PhantomData<T>,
    pub(crate) order_by: Option<OrderBy>,
}

pub trait Saveable {
    // TODO: customize error type
    fn save(&self) -> Result<(), String>;
}

pub trait Deleteable {
    // TODO: rework this to actually Delete, not get_delete_statement
    fn get_delete_statement(&self) -> DeleteStatement<Self>
    where
        Self: std::marker::Sized;
}

pub trait Updateable {
    fn get_update_statement(&self) -> UpdateStatement
    where
        Self: std::marker::Sized;
}

pub struct OrderBy {
    col_name: Identifier,
    direction: OrderDirection,
}
pub enum OrderDirection {
    Ascending,
    Desending,
}
impl Display for OrderDirection {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            OrderDirection::Ascending => f.write_str("ASC"),
            OrderDirection::Desending => f.write_str("DESC"),
        }
    }
}

impl<T> Query<T> {
    #[allow(unused)]
    pub fn limit(
        mut self,
        limit: usize,
    ) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn filter(
        mut self,
        filter: Filter,
    ) -> Self {
        if let Some(f) = self.filter {
            self.filter = Some(f & filter)
        } else {
            self.filter = Some(filter)
        }
        self
    }

    pub fn order_by(
        mut self,
        col_name: Identifier,
        direction: OrderDirection,
    ) -> Self {
        self.order_by = Some(OrderBy {
            col_name,
            direction,
        });
        self
    }
}

pub trait Insertable
where
    Self: Sized,
{
    type CreateRequest: Default + Serialize + for<'a> Deserialize<'a> + Into<Self>;

    fn get_insert_statement(&self) -> InsertStatement;
}

impl<T> BuildSql for Query<T> {
    // #[cfg(not(Bfeature = "sql_json"))]
    // fn build_sql(
    //     &self,
    //     query_builder: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    // ) {
    // }
    // #[cfg(feature = "sql_json")]
    fn build_sql(
        &self,
        query_builder: &mut sqlx::QueryBuilder<'_, sqlx::Postgres>,
    ) {
        let table_name = self.table.table_name.clone();
        let mut group_by: Vec<String> = vec![format!("{}.id", &self.table.table_name)];

        // STEP ONE: get all table relationships
        let JsonBuildObjectResponse {
            sql,
            group_by: mut addtl_group_by,
        } = self.table.json_build_object("");
        group_by.append(&mut addtl_group_by);

        // WITH  one_to_many_child__json
        query_builder.push(r"SELECT ");
        query_builder.push(sql);
        query_builder.push(" json_result");
        query_builder.push(" FROM ");
        query_builder.push(&table_name);

        for join_stmt in self.table.get_join_tables("") {
            query_builder.push(" ");
            query_builder.push(&join_stmt);
            query_builder.push(" ");
        }
        if let Some(filter) = &self.filter {
            query_builder.push(" WHERE ");
            filter.build_sql(query_builder);
        }
        // TODO: Unhack (part of the "everything built on id" problem)
        query_builder.push(" GROUP BY (").push(group_by.join(", ")).push(")");
        if let Some(OrderBy {
            col_name,
            direction,
        }) = &self.order_by
        {
            query_builder.push(format!(" ORDER BY {col_name} {direction} "));
        }
        if let Some(limit) = &self.limit {
            query_builder.push(format!(" LIMIT {limit} "));
        }

        // STEP FOUR: Probably will need to do more with build_query_as. will find out
    }
}

#[cfg(test)]
mod tests {
    use crate::data_definition::table::{DatabaseTableDefinition, TableColumn};

    fn get_table_def() -> DatabaseTableDefinition {
        type T = TableColumn;

        DatabaseTableDefinition::new("table_2")
            .unwrap()
            .column(T::string("string_nullable").unwrap())
            .column(T::bool("bool").unwrap().non_null())
            .column(T::int("int").unwrap().non_null())
            .column(T::float("float").unwrap().non_null())
            .column(T::timestamp("timestamp").unwrap().non_null())
            .column(T::uuid("id").unwrap().non_null())
            .into()
    }
    #[test]
    fn test_queries() {
        const _EXPECTED_QUERY: &str = "SELECT * FROM item INNER JOIN sub_item ON sub_item.parent_id = item.id WHERE sub_item.name like 'BUG%';";

        let _table_def: DatabaseTableDefinition = get_table_def();
        // let _query = Query::<String

        todo!()
    }
}
