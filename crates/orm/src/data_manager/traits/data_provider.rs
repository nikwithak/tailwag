use std::ops::{Deref, DerefMut};

use crate::queries::filterable_types::Filterable;

/// Provides basic CRUD operations for a type's data source
///
/// Unless you have a specific use case that warrants a custom implementation,
/// it is recommended that you use one of the premade DataProviders for typical
/// use cases (examples below). A long-term goal is to define the configurability
/// of these to handle contexts / user permissions / etc.
///
///  - [x] PostgresDataProvider
///  - [x] TODO: RestApiDataProvider
///  - [x] TODO: InMemoryDataProvider // Intended for debugging/development
///                                   // primarily, but may have other uses (e.g. used in caching)
///  - [ ] TODO: LocalFileDataProvider
///  - [ ] TODO: S3DataProvider
///  - [ ] TODO: CachedDataProvider<P: DataProvider>
///  - { } TODO: PolicyEnforcedDataProvider
///  - [ ] TODO: MultiSourceDataProvider
#[allow(async_fn_in_trait)]
pub trait DataProvider<T>
where
    T: Filterable,
    Self: Sized,
{
    type CreateRequest: Default;

    // fn all(&self) -> Vec<T>;
    async fn all(&self) -> Result<impl Iterator<Item = T>, crate::Error>;
    async fn get(
        &self,
        predicate: impl Fn(T::FilterType) -> crate::queries::Filter,
    ) -> Result<Option<T>, crate::Error>;
    async fn create(
        &self,
        item: Self::CreateRequest,
    ) -> Result<T, crate::Error>;
    async fn delete(
        &self,
        item: T, // You give it up when you ask to delete it!
    ) -> Result<(), crate::Error>;
    async fn update(
        &self,
        item: &T,
    ) -> Result<(), crate::Error>;
}

pub trait WithFilter<T>
where
    T: Filterable,
{
    type R;
    fn with_filter(
        &self,
        predicate: impl Fn(T::FilterType) -> crate::queries::Filter,
    ) -> Self::R;
}

pub struct Provided<'a, T, P>
where
    P: DataProvider<T>,
    T: Filterable,
{
    provider: &'a P,
    data: T,
}

impl<'a, T, P> DerefMut for Provided<'a, T, P>
where
    P: DataProvider<T>,
    T: Filterable,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<'a, T, P> Deref for Provided<'a, T, P>
where
    P: DataProvider<T>,
    T: Filterable,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[allow(unused)]
impl<'a, T, P> Provided<'a, T, P>
where
    P: DataProvider<T>,
    T: Filterable,
{
    async fn save(&self) -> Result<(), crate::Error> {
        self.provider.update(&self.data).await
    }

    async fn delete(self) -> Result<(), crate::Error> {
        self.provider.delete(self.data).await
    }
}
