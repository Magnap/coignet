use std::sync::Weak;
use std::sync::{Arc, RwLock};
use lru_cache::LruCache;
use failure::{error_msg, Error};
use std::hash::Hash;

pub struct Store<K: Hash + Eq, V> {
    pub ptr: Arc<RwLock<CachedStore<K, Arc<V>>>>,
}

impl<K: Hash + Eq, V> Clone for Store<K, V> {
    fn clone(&self) -> Self {
        Store {
            ptr: self.ptr.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Cached<K: Eq + Hash, V> {
    ptr: Weak<V>,
    pub key: K,
    store: Store<K, V>,
}

impl<K: Eq + Hash + Clone, V> PartialEq for Cached<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl<K: Eq + Hash + Clone, V> Eq for Cached<K, V> {}

pub struct CachedStore<K: Eq + Hash, V> {
    pub cache: LruCache<K, V>,
    pub lookup: Box<FnMut(&K) -> Result<V, Error> + Send>,
}

impl<K: Eq + Hash, V> CachedStore<K, V> {
    pub fn new(cache_size: usize, lookup: Box<FnMut(&K) -> Result<V, Error> + Send>) -> Self {
        CachedStore {
            lookup: lookup,
            cache: LruCache::new(cache_size),
        }
    }
}

impl<K: Eq + Hash + Clone, V> CachedStore<K, V> {
    fn lookup(&mut self, key: &K) -> Result<&mut V, Error> {
        if !self.cache.contains_key(key) {
            let value = (self.lookup)(key)?;
            self.cache.insert(key.clone(), value);
        }
        Ok(self.cache.get_mut(key).ok_or_else(|| {
            error_msg(
                "Value disappeared immediately after inserting into cache. Maybe cache has size 0?",
            )
        })?)
    }
}

impl<K: Eq + Hash, V> Store<K, V> {
    pub fn new(store: CachedStore<K, Arc<V>>) -> Self {
        Store {
            ptr: Arc::new(RwLock::new(store)),
        }
    }
    pub fn new_ref(&self, key: K) -> Cached<K, V> {
        Cached::new(self.clone(), key)
    }
}

impl<K: Eq + Hash, V> Cached<K, V> {
    pub fn new(store: Store<K, V>, key: K) -> Self {
        Cached {
            ptr: Weak::new(),
            key,
            store,
        }
    }
}

impl<K: Eq + Hash + Clone, V> Cached<K, V> {
    pub fn fetch(&self) -> Result<Arc<V>, Error> {
        Ok(match self.ptr.upgrade() {
            Some(arc) => arc,
            None => self.store
                .ptr
                .write()
                .expect("Store RwLock poisoned")
                .lookup(&self.key)?
                .clone(),
        })
    }
}
