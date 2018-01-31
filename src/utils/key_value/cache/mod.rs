use super::KeyValueStore;
use std::sync::{Arc, Mutex};

// Could possibly be made into a trait alias of KeyValueStore somehow
pub trait Cache<K, V> {
    fn insert(&mut self, key: K, value: V);
    fn lookup(&mut self, key: &K) -> Option<&V>;
    fn delete(&mut self, key: &K);
}

pub struct Cached<C, KVS> {
    pub store: KVS,
    cache: Arc<Mutex<C>>,
}

impl<'a, K: Clone, V, KVS: KeyValueStore<'a, K, V>, C: Cache<K, KVS::Entry>> KeyValueStore<'a, K, V>
    for Cached<C, KVS>
where
    KVS::Entry: Clone,
{
    type Error = KVS::Error;
    type Entry = KVS::Entry;

    fn insert(&'a mut self, key: K, value: V) -> Result<Self::Entry, Self::Error> {
        let entry = self.store.insert(key.clone(), value)?;
        {
            let mut cache = self.cache.lock().expect("cache mutex poisoned");
            cache.insert(key, entry.clone());
        }
        Ok(entry)
    }
    fn lookup(&'a mut self, key: &'a K) -> Result<Self::Entry, Self::Error> {
        let mut cache = self.cache.lock().expect("cache mutex poisoned");
        let entry;
        {
            entry = cache.lookup(key).cloned();
        }
        let entry = match entry {
            Some(entry) => entry,
            None => {
                let entry = self.store.lookup(key)?;
                cache.insert(key.clone(), entry.clone());
                entry
            }
        };
        Ok(entry)
    }
    fn delete(&'a mut self, key: &'a K) -> Result<(), Self::Error> {
        let mut cache = self.cache.lock().expect("cache mutex poisoned");
        cache.delete(key);
        self.store.delete(key)
    }
}
