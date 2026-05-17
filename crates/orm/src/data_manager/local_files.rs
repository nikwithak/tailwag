use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    data_definition::table::DatabaseTableDefinition,
    data_manager::traits::DataProvider,
    queries::{filterable_types::Filterable, Filter},
};
use std::{fs, marker::PhantomData, path::PathBuf};

use super::rest_api::Id;

/// Stores data objects as JSON files on local disk. Intended for local development in lieu of PostgresDataProviders.
/// Very loosely tested - not recommended for Production use.
#[derive(Clone)]
pub struct LocalFileDataProvider<T> {
    pub table_definition: DatabaseTableDefinition,
    pub root_folder_path: PathBuf,
    pub _t: PhantomData<T>,
}

impl<T: Id> LocalFileDataProvider<T> {
    fn get_filepath(
        &self,
        item_id: &Uuid,
    ) -> PathBuf {
        let filename = format!("{}.json", item_id);
        let mut path = self.root_folder_path.clone();
        path.push(&filename);
        path.to_owned()
    }
}

impl<T: Default + Sync + Send + Id + Serialize + for<'a> Deserialize<'a> + Filterable>
    DataProvider<T> for LocalFileDataProvider<T>
{
    type CreateRequest = T;

    #[allow(unreachable_code)]
    async fn all(&self) -> Result<impl Iterator<Item = T>, crate::Error> {
        todo!();
        Ok(Vec::new().into_iter())
    }

    async fn create(
        &self,
        item: Self::CreateRequest,
    ) -> Result<T, crate::Error> {
        // let item: T = item.into();

        let path = self.get_filepath(item.id());
        let contents = serde_json::to_string(&item).unwrap();
        std::fs::write(path, contents).unwrap();

        Ok(item)
    }

    async fn get(
        &self,
        _predicate: impl FnOnce(<T as Filterable>::FilterType) -> Filter,
    ) -> Result<Option<T>, crate::Error> {
        todo!()
        // let path = self.get_filepath(&id);
        // let contents = std::fs::read_to_string(path).unwrap();
        // match serde_json::from_str::<T>(&contents) {
        //     Ok(a) => Ok(Some(a)),
        //     Err(e) => Err(format!("{:?}", &e))?,
        // }
    }

    async fn delete(
        &self,
        item: T,
    ) -> Result<(), crate::Error> {
        let path = self.get_filepath(item.id());
        // // For safety, make sure it's the right object:
        // self.get(*item.id()).await?;
        fs::remove_file(path)?;
        Ok(())
    }

    async fn update(
        &self,
        item: &T,
    ) -> Result<(), crate::Error> {
        let path = self.get_filepath(item.id());
        let contents = serde_json::to_string(item).unwrap();
        std::fs::write(path, contents).unwrap();
        Ok(())
    }
}
