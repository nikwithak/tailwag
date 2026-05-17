use std::{collections::HashMap, hash::Hash};

pub trait GetOrDefault<K, V>
where
    K: Hash + Eq + PartialEq,
{
    fn get_or_default(
        &mut self,
        key: &K,
    ) -> &V;
    fn get_or_default_mut(
        &mut self,
        key: &K,
    ) -> &mut V;
}

impl<K, V> GetOrDefault<K, V> for HashMap<K, V>
where
    K: Hash + Eq + PartialEq + Clone,
    V: Default,
{
    fn get_or_default(
        &mut self,
        key: &K,
    ) -> &V {
        if !self.contains_key(key) {
            self.insert(key.clone(), Default::default());
        }
        self.get(key).expect("Value must be available at this point.")
    }

    fn get_or_default_mut(
        &mut self,
        key: &K,
    ) -> &mut V {
        if !self.contains_key(key) {
            self.insert(key.clone(), Default::default());
        }
        self.get_mut(key).expect("Value must be available at this point.")
    }
}

pub trait GetOrInsert<K, V>
where
    K: Hash + Eq + PartialEq,
{
    fn get_or_insert(
        &mut self,
        key: &K,
        val: V,
    ) -> &V;
    fn get_or_insert_mut(
        &mut self,
        key: &K,
        val: V,
    ) -> &mut V;
}
impl<K, V> GetOrInsert<K, V> for HashMap<K, V>
where
    K: Hash + Eq + PartialEq + Clone,
{
    fn get_or_insert(
        &mut self,
        key: &K,
        default_val: V,
    ) -> &V {
        if !self.contains_key(key) {
            self.insert(key.clone(), default_val);
        }
        self.get(key).expect("Value must be available at this point.")
    }

    fn get_or_insert_mut(
        &mut self,
        key: &K,
        default_val: V,
    ) -> &mut V {
        if !self.contains_key(key) {
            self.insert(key.clone(), default_val);
        }
        self.get_mut(key).expect("Value must be available at this point.")
    }
}
