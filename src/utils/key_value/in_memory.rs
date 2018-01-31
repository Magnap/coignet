use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

pub struct InMemory<K, V>(HashMap<K, Entry<V>>);

pub struct Entry<T>(Arc<RwLock<T>>);
impl<T> Clone for Entry<T> {
    fn clone(&self) -> Self {
        Entry(self.0.clone())
    }
}

impl<K: Eq + Hash, V> InMemory<K, V> {
    pub fn new() -> Self {
        InMemory(HashMap::new())
    }
}

impl<V> super::Entry<V> for Entry<V> {
    type Error = !;

    fn read<T, F: FnOnce(&V) -> T>(&self, fun: F) -> Result<T, Self::Error> {
        let value = self.0.read().unwrap();
        Ok(fun(&value))
    }
    fn write<T, F: FnOnce(&mut V) -> T>(&mut self, fun: F) -> Result<T, Self::Error> {
        let mut value = self.0.write().unwrap();
        Ok(fun(&mut value))
    }
}

impl<'a, K: Hash + Eq, V> super::KeyValueStore<'a, K, V> for InMemory<K, V> {
    type Error = ();
    type Entry = Entry<V>;

    fn insert(&mut self, key: K, value: V) -> Result<Self::Entry, Self::Error> {
        let entry = Entry(Arc::new(RwLock::new(value)));
        self.0.insert(key, entry.clone());
        Ok(entry)
    }
    fn lookup(&mut self, key: &K) -> Result<Self::Entry, Self::Error> {
        match self.0.get(key) {
            Some(entry) => Ok(entry.clone()),
            None => Err(()),
        }
    }
    fn delete(&mut self, key: &K) -> Result<(), Self::Error> {
        self.0.remove(key);
        Ok(())
    }
}
