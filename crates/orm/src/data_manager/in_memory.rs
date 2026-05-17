use crate::{
    data_manager::{rest_api::Id, traits::DataProvider},
    queries::filterable_types::Filterable,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

/// InMemoryDataProvider - Wraps a HashMap in the DatProvider interface.
/// Status: Prototype / Hacky
/// Notes: Lots of data duplication, not memory efficient, and blocks on reads due to the Arc<Mutex>, which is required to fit the trait signtatures and enable Clone

#[derive(Clone, Default)]
pub struct InMemoryDataProvider<T> {
    items: Arc<Mutex<HashMap<Uuid, T>>>,
}

impl<T: Clone + Id + Send + Default + Filterable> DataProvider<T> for InMemoryDataProvider<T> {
    type CreateRequest = T;

    async fn all(&self) -> Result<impl Iterator<Item = T>, crate::Error> {
        // todo!();
        // Ok(vec![].into_iter())
        let items = self.items.lock().map_err(|e| crate::Error::MutexLock(e.to_string()))?;
        Ok(items.values().map(|v| v.to_owned()).collect::<Vec<_>>().into_iter())
    }

    async fn get(
        &self,
        _predicate: impl Fn(T::FilterType) -> crate::queries::Filter,
    ) -> Result<Option<T>, crate::Error> {
        let _items = self.items.lock().unwrap();
        // let item = items.get(&id).cloned();
        // Ok(item)
        todo!()
    }

    async fn create(
        &self,
        item: Self::CreateRequest,
    ) -> Result<T, crate::Error> {
        let mut items = self.items.lock().unwrap();
        let id = item.id();
        if items.contains_key(id) {
            Err(format!("Already contains object with id ({})", id))?
        } else {
            items.insert(*item.id(), item.clone());
            Ok(item)
        }
    }

    async fn delete(
        &self,
        item: T,
    ) -> Result<(), crate::Error> {
        let mut items = self.items.lock().unwrap();
        match items.remove(item.id()) {
            Some(_) => Ok(()),
            None => Err("Item not found".to_string())?,
        }
    }

    async fn update(
        &self,
        item: &T,
    ) -> Result<(), crate::Error> {
        let mut items = self.items.lock().unwrap();
        let id = item.id();
        if items.contains_key(id) {
            items.insert(*item.id(), item.clone());
            Ok(())
        } else {
            Err(format!("no object with id ({})", id))?
        }
    }
}
