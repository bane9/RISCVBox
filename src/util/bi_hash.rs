use std::collections::HashMap;
use std::hash::Hash;

pub struct BiMap<K, V> {
    forward: HashMap<K, V>,
    reverse: HashMap<V, K>,
}

impl<K, V> BiMap<K, V>
where
    K: Eq + Hash + Clone + Copy,
    V: Eq + Hash + Clone + Copy,
{
    pub fn new() -> Self {
        BiMap {
            forward: HashMap::new(),
            reverse: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<(K, V)>
    where
        K: Eq + Hash + Clone + Copy,
        V: Eq + Hash + Clone + Copy,
    {
        let old_key_value = self.forward.insert(key.clone(), value.clone());
        let old_value_key = self.reverse.insert(value.clone(), key.clone());

        if let Some(old_value) = old_key_value {
            self.reverse.remove(&old_value);
        }

        if let Some(old_key) = old_value_key {
            self.forward.remove(&old_key);
        }

        old_value_key.map(|old_key| (old_key, value))
    }

    pub fn get_by_key(&self, key: K) -> Option<&V> {
        self.forward.get(&key)
    }

    pub fn get_by_value(&self, value: V) -> Option<&K> {
        self.reverse.get(&value)
    }

    pub fn remove_by_key(&mut self, key: &K) -> Option<V> {
        let value = self.forward.remove(key);

        if let Some(value) = value {
            self.reverse.remove(&value);
        }

        value
    }

    pub fn remove_by_value(&mut self, value: &V) -> Option<K> {
        let key = self.reverse.remove(value);

        if let Some(key) = key {
            self.forward.remove(&key);
        }

        key
    }
}
