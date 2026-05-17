use crate::queries::filterable_types::WithPrefix;
use crate::{
    data_definition::table::{DatabaseTableDefinition, Identifier},
    migration::Migration,
    queries::{
        filterable_types::Filterable, Deleteable, Filter, Insertable, OrderDirection, Query,
        Updateable,
    },
    BuildSql,
};
use sqlx::Row;
use sqlx::{Error, Execute, Pool, Postgres, QueryBuilder};
use std::{marker::PhantomData, sync::Arc};

use super::{rest_api::Id, traits::WithFilter};

#[derive(Clone)]
pub struct PostgresDataProvider<T: Insertable> {
    pub table_definition: Arc<DatabaseTableDefinition>,
    pub db_pool: Pool<Postgres>,
    pub _t: PhantomData<T>,
    pub default_filter: Option<Filter>,
    // pub(crate) parent_data_system:
}

impl<T> PostgresDataProvider<T>
where
    T: Insertable + Filterable,
{
    pub fn with_default_filter<F>(
        mut self,
        filter: F,
    ) -> Self
    where
        F: Fn(T::FilterType) -> Filter,
    {
        // TODO: Figure out how to restrict this, without making it "pub". "pub" allows a malicious endpoint to strip the filter.
        let filter = filter(T::FilterType::with_prefix(""));
        self.default_filter = Some(filter);
        self
    }
}

impl<T> PostgresDataProvider<T>
where
    T: Insertable
        + Deleteable
        + Updateable
        + Send
        + serde::Serialize
        + for<'d> serde::Deserialize<'d>
        + Clone
        + Unpin
        + Id
        // + Serialize
        + Filterable
        + Default,
{
    pub async fn get_many(
        &self,
        predicate: impl Fn(<T as Filterable>::FilterType) -> crate::queries::Filter,
    ) -> Result<impl Iterator<Item = T>, crate::Error> {
        let query = self.new_query().with_filter(predicate);
        Ok(query.execute().await?.into_iter())
    }
}

impl<T> PostgresDataProvider<T>
where
    T: Insertable,
{
    pub fn new(
        table_definition: Arc<DatabaseTableDefinition>,
        db_pool: Pool<Postgres>,
    ) -> Self {
        Self {
            table_definition,
            db_pool,
            _t: PhantomData,
            default_filter: None,
        }
    }

    pub fn new_query(&self) -> ExecutableQuery<T> {
        let query = Query::<T> {
            table: self.table_definition.clone(),
            filter: self.default_filter.clone(),
            limit: None,
            order_by: None,
            _t: Default::default(),
        };

        ExecutableQuery {
            query,
            db_pool: self.db_pool.clone(),
        }
    }
}

pub trait GetTableDefinition {
    fn get_table_definition() -> DatabaseTableDefinition
    where
        Self: std::marker::Sized;
}

impl<T: GetTableDefinition> GetTableDefinition for Option<T> {
    fn get_table_definition() -> DatabaseTableDefinition
    where
        Self: std::marker::Sized,
    {
        T::get_table_definition()
    }
}

impl<T: GetTableDefinition> GetTableDefinition for Vec<T> {
    fn get_table_definition() -> DatabaseTableDefinition
    where
        Self: std::marker::Sized,
    {
        T::get_table_definition()
    }
}

/// Wraps a `Query<T>` alongside a DB Pool (using sqlx), to enable ergonomic querying.
pub struct ExecutableQuery<T> {
    query: Query<T>,
    db_pool: Pool<Postgres>,
}

// impl<T: Insertable> Deref for ExecutableQuery<T> {
//     type Target = Query<T>;
//     fn deref(&self) -> &Self::Target {
//         &self.query
//     }
// }

// impl<T: Insertable> DerefMut for ExecutableQuery<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.query
//     }
// }

impl<T> From<ExecutableQuery<T>> for Vec<T>
where
    T: Insertable + for<'r> serde::Deserialize<'r> + Send + Unpin,
{
    fn from(val: ExecutableQuery<T>) -> Self {
        futures::executor::block_on(val.execute()).unwrap()
    }
}

impl<T: Insertable + for<'d> serde::Deserialize<'d> + Send + Unpin> ExecutableQuery<T> {
    pub async fn execute(self) -> Result<Vec<T>, Error> {
        // We wrap the whole thing in a `to_json` on the DB side. This makes it supes easy to deserialize.
        // Without his, it got really messy, because it seems that SQLX doesn't support nested deserialization on its own.
        // A little bit more overhead, perhaps, but jeeeeez does it save on dvelopment. And postgres is probably
        // not the limiting factor rn anyway.
        // let mut query_builder = QueryBuilder::new("SELECT to_json(r) as json_result FROM (");
        let mut query_builder = QueryBuilder::new("");
        self.query.build_sql(&mut query_builder);
        // query_builder.push(") r");

        log::debug!("SQL query: {}", query_builder.sql());
        let result = query_builder.build().fetch_all(&self.db_pool).await?;
        let results = result.into_iter().map(|row| {
            let rowresult: serde_json::Value = row.get("json_result");
            log::debug!("Result: {:?}", rowresult.to_string());
            serde_json::from_value::<T>(rowresult)
        });
        let results: Result<_, serde_json::Error> = results.collect();
        results.map_err(|e| Error::Decode(Box::new(e)))
    }
}

impl<T: Filterable> ExecutableQuery<T> {
    pub fn with_filter<F>(
        mut self,
        derive_filter: F,
    ) -> Self
    where
        F: Fn(T::FilterType) -> Filter,
    {
        let filter = derive_filter(T::FilterType::with_prefix(""));
        self.query = self.query.filter(filter);
        self
    }

    pub fn order_by(
        mut self,
        col_name: Identifier,
        direction: OrderDirection,
    ) -> Self {
        self.query = self.query.order_by(col_name, direction);
        self
    }

    pub fn order_asc(
        self,
        col_name: Identifier,
    ) -> Self {
        self.order_by(col_name, OrderDirection::Ascending)
    }
    pub fn order_desc(
        self,
        col_name: Identifier,
    ) -> Self {
        self.order_by(col_name, OrderDirection::Desending)
    }
    pub fn limit(
        mut self,
        limit: usize,
    ) -> Self {
        self.query = self.query.limit(limit);
        self
    }
}

// Migration Handling
impl<T: Insertable> PostgresDataProvider<T>
where
    T: Clone + Send + std::fmt::Debug + 'static + for<'d> serde::Deserialize<'d>,
{
    pub fn build_migration(&self) -> Option<Migration> {
        Migration::compare(
            None, // TODO: Need to get the old migration
            vec![self.table_definition.clone()],
        );
        todo!()
    }
    pub async fn run_migrations(&self) -> Result<(), String> {
        log::info!("[DATABASE] Running Migrations");
        let migration = self.build_migration();
        if let Some(migration) = migration {
            let mut sql = sqlx::QueryBuilder::new("");
            migration.build_sql(&mut sql);

            let sql = sql.into_sql();
            let mut transaction = self.db_pool.begin().await.unwrap();
            match sqlx::query(&sql).execute(&mut *transaction).await {
                Ok(_) => {},
                Err(e) => {
                    log::error!("Failed to run migrations");
                    return Err(format!("{}", e));
                },
            }
            transaction.commit().await.unwrap();
        } else {
            log::info!("[DATABASE] No Migrations");
        }
        Ok(())
    }
}

impl<T> super::traits::DataProvider<T> for PostgresDataProvider<T>
where
    T: Insertable
        + Deleteable
        + Updateable
        + Send
        + serde::Serialize
        + for<'d> serde::Deserialize<'d>
        + Clone
        + Unpin
        + Id
        // + Serialize
        + Filterable
        + Default,
{
    type CreateRequest = T::CreateRequest; // TODO: Implement this based on the implementaiton of Insertable?

    async fn get(
        &self,
        predicate: impl Fn(<T as Filterable>::FilterType) -> crate::queries::Filter,
    ) -> Result<Option<T>, crate::Error> {
        let query = self.new_query().with_filter(predicate).limit(2);
        let mut results = query.execute().await?;
        if results.len() > 1 {
            return Err(crate::Error::DataIntegrity("Multiple items found for ID {}".to_string()));
        }
        Ok(results.pop())
    }

    async fn all(&self) -> Result<impl Iterator<Item = T>, crate::Error> {
        let query = self.new_query();
        Ok(query.execute().await?.into_iter())
    }

    async fn create(
        &self,
        item: Self::CreateRequest,
    ) -> Result<T, crate::Error> {
        let item = item.into();
        let insert_statement = item.get_insert_statement();

        let pool = self.db_pool.clone();
        let mut transaction = pool.begin().await?;
        let mut builder: QueryBuilder<'_, Postgres> = QueryBuilder::new("");
        insert_statement.build_sql(&mut builder);

        builder.build().execute(&mut *transaction).await?;
        transaction.commit().await?;

        Ok(item)
    }

    async fn delete(
        &self,
        item: T,
    ) -> Result<(), crate::Error> {
        let mut builder: QueryBuilder<'_, Postgres> = sqlx::QueryBuilder::new("");
        let mut delete_stmt = item.get_delete_statement();
        if let Some(filter) = &self.default_filter {
            delete_stmt = delete_stmt.with_filter(filter.clone());
        }
        delete_stmt.build_sql(&mut builder);
        let query = builder.build();
        &query.sql();
        if query.execute(&self.db_pool).await?.rows_affected() > 1 {
            panic!("Deleted more than one row in a Delete operation. This should not happen.");
        }

        Ok(())
    }

    async fn update(
        &self,
        item: &T,
    ) -> Result<(), crate::Error> {
        // TODO: No default_filter checks in place here!!
        let mut transaction = self.db_pool.begin().await?;
        let update_statement = item.get_update_statement();

        let mut builder: QueryBuilder<'_, Postgres> = QueryBuilder::new("");
        update_statement.build_sql(&mut builder);

        builder.build().execute(&mut *transaction).await?;
        transaction.commit().await?;

        Ok(())
    }
}

impl<T> WithFilter<T> for PostgresDataProvider<T>
where
    T: Filterable + Insertable + Clone,
{
    type R = ExecutableQuery<T>;
    fn with_filter(
        &self,
        predicate: impl Fn(<T as Filterable>::FilterType) -> crate::queries::Filter,
    ) -> Self::R {
        let query = Query::<T> {
            table: self.table_definition.clone(),
            filter: Some(predicate(T::FilterType::with_prefix(""))),
            limit: None,
            order_by: None,
            _t: Default::default(),
        };
        ExecutableQuery {
            query,
            db_pool: self.db_pool.clone(),
        }
    }
}

pub trait BuildDataProvider
where
    Self: Sized + Insertable,
{
    fn build_data_provider(&self) -> PostgresDataProvider<Self>;
}

impl<T> BuildDataProvider for T
where
    T: Sized + Insertable,
{
    fn build_data_provider(&self) -> PostgresDataProvider<Self> {
        todo!()
    }
}
