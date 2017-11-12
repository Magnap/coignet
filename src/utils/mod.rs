pub mod cached;
pub mod persisted;

use failure::{Error, Fail};
pub fn error_type<T: Fail>(e: Error) -> Result<(), Error> {
    e.downcast::<T>()?;
    Ok(())
}

pub mod in_memory {
    use super::cached::{Arc, CachedStore, Hash, LruCache, RwLock};
    use failure::error_msg;
    use std::collections::HashMap;

    pub type InMemoryStore<K, V> = CachedStore<K, Arc<V>>;
    impl<K: Eq + Hash + Sync + Send + 'static, V: Sync + Send + Clone + 'static> InMemoryStore<K, V> {
        pub fn new(cache_size: usize) -> (Self, Arc<RwLock<HashMap<K, V>>>) {
            let store = Arc::new(RwLock::new(HashMap::new()));
            let cached_store;
            {
                let store = store.clone();
                cached_store = CachedStore {
                    lookup: Box::new(move |key| {
                        let store = store.read().unwrap();
                        let value: &V = store.get(key).ok_or_else(|| error_msg("Key not found"))?;
                        Ok(Arc::new(value.clone()))
                    }),
                    cache: LruCache::new(cache_size),
                };
            }
            (cached_store, store)
        }
    }
}
