use lru;
use std::hash::Hash;
use super::Cache;

pub struct LruCache<K, V>(lru::LruCache<K, V>);

impl<K: Hash + Eq, V> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        LruCache(lru::LruCache::new(capacity))
    }
}

impl<K: Hash + Eq, V> Cache<K, V> for LruCache<K, V> {
    fn insert(&mut self, key: K, value: V) {
        self.0.put(key, value)
    }
    fn lookup(&mut self, key: &K) -> Option<&V> {
        self.0.get(key)
    }
    fn delete(&mut self, key: &K) {
        self.0.pop(key);
    }
}
