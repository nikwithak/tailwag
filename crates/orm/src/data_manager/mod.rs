pub mod local_storage_provider;
mod postgres;
pub use postgres::*;
use rest_api::Id;
use serde::{Deserialize, Serialize};
use traits::DataProvider;

use crate::queries::{self, filterable_types::Filterable, Insertable};
pub mod rest_api;
pub mod threaded;
pub mod traits;

pub mod in_memory;
pub mod local_files;
// struct PostgresDataProvider {}
// struct FileS3DataProvider {}
// struct FileLocalStorageProvider {}
// struct MongoDBDataProvider {}

#[allow(unused)]
pub enum DataProviderType<T: Insertable> {
    Postgres(PostgresDataProvider<T>),
    // InMemory(InMemoryDataProvider<T>),
    // JsonFiles(LocalFileDataProvider<T>),
    // Files(PostgresDataProvider<T>),
}

#[allow(unused)]
struct DataManager<T>
where
    T: Filterable + Insertable,
{
    inner: DataProviderType<T>,
}

impl<T: Filterable + Insertable> DataManager<T> {}
#[allow(unused)]
enum DataManagerError {
    Error(String),
}

impl<T: Filterable + Insertable> DataProvider<T> for DataManager<T>
where
    T: Clone
        + Default
        + for<'d> Deserialize<'d>
        + Id
        + Serialize
        + Send
        + Unpin
        + queries::query_builder::Deleteable
        + queries::query_builder::Updateable
        + std::marker::Send,
{
    type CreateRequest = <T as Insertable>::CreateRequest; // TODO - This is just placeholder

    async fn all(&self) -> Result<impl Iterator<Item = T>, crate::Error> {
        match &self.inner {
            DataProviderType::Postgres(dp) => dp.all().await,
            // DataProviderType::InMemory(dp) => dp.all().await,
            // DataProviderType::JsonFiles(dp) => todo!(),
            // DataProviderType::Files(dp) => todo!(),
        }
    }

    async fn get(
        &self,
        _predicate: impl Fn(T::FilterType) -> crate::queries::Filter,
    ) -> Result<Option<T>, crate::Error> {
        todo!()
    }

    async fn create(
        &self,
        _item: Self::CreateRequest,
    ) -> Result<T, crate::Error> {
        todo!()
    }

    async fn delete(
        &self,
        _item: T, // You give it up when you ask to delete it!
    ) -> Result<(), crate::Error> {
        todo!()
    }

    async fn update(
        &self,
        _item: &T,
    ) -> Result<(), crate::Error> {
        todo!()
    }
}
