use std::sync::{Arc, Mutex};

use crate::queries::Insertable;

use super::PostgresDataProvider;

/// ThreadedDataManager? Or Cursor?
///
/// Goal: Be able to use this as a local cache AND a way of updating remotely.
pub struct ThreadedDataManager<T>
where
    T: Insertable,
{
    _items: Arc<Mutex<Vec<T>>>,
    _data_provider: PostgresDataProvider<T>,
}
