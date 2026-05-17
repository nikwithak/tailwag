use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
    time::Instant,
};

pub struct QueuedBTreeMap<K, V> {
    map: BTreeMap<K, (V, Instant)>,
}

impl<K, V> Deref for QueuedBTreeMap<K, V> {
    type Target = BTreeMap<K, (V, Instant)>;

    fn deref(&self) -> &Self::Target {
        let time = std::time::Instant::now();
        &self.map
    }
}

impl<K, V> DerefMut for QueuedBTreeMap<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
