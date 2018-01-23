pub mod cached;
pub mod persisted;

use failure::{Error, Fail};
pub fn error_type<T: Fail>(e: Error) -> Result<(), Error> {
    e.downcast::<T>()?;
    Ok(())
}

pub mod in_memory {
    use super::cached::CachedStore;
    use std::hash::Hash;
    use std::sync::{Arc, RwLock};
    use failure::err_msg;
    use std::collections::HashMap;

    pub type InMemoryStore<K, V> = CachedStore<K, Arc<V>>;
    pub fn new<K: Hash + Eq + Sync + Send + 'static, V: Clone + Sync + Send + 'static>(
        cache_size: usize,
    ) -> (InMemoryStore<K, V>, Arc<RwLock<HashMap<K, V>>>) {
        let store = Arc::new(RwLock::new(HashMap::new()));
        let cached_store;
        {
            let store = store.clone();
            cached_store = CachedStore::new(
                cache_size,
                Box::new(move |key| {
                    let store = store.read().unwrap();
                    let value: &V = store.get(key).ok_or_else(|| err_msg("Key not found"))?;
                    Ok(Arc::new(value.clone()))
                }),
            );
        }
        (cached_store, store)
    }
}

pub mod file_store {
    use super::cached::{Cached, CachedStore, Store};
    use super::persisted::Persisted;
    use std::hash::Hash;
    use std::sync::{Arc, Mutex};
    use failure::Error;
    use std::path::PathBuf;
    use std::io::BufReader;
    use std::fs::File;
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use bincode::{deserialize_from, serialize_into, Infinite};

    pub struct FileStore<K: Hash + Eq + ToString, V: Serialize + DeserializeOwned> {
        dir: PathBuf,
        cache: Store<K, Persisted<V>>,
    }

    pub type FileStored<K, V> = Cached<K, Persisted<V>>;

    impl<K: Hash + Eq + ToString, V: Serialize + DeserializeOwned> FileStore<K, V> {
        pub fn new(dir: PathBuf, cache_size: usize) -> Self {
            let cache;
            {
                let dir = dir.clone();
                cache = Store::new(CachedStore::new(
                    cache_size,
                    Box::new(move |key| {
                        let mut loc = dir.clone();
                        loc.push(&ToString::to_string(key));
                        let mut file = BufReader::new(File::open(loc)?);
                        let value = deserialize_from(&mut file, Infinite)?;
                        let persisted;
                        {
                            let dir = dir.clone();
                            let key: PathBuf = ToString::to_string(key).into();
                            let persist = Box::new(move |val: &V| {
                                let mut loc = dir.clone();
                                loc.push(&key);
                                let mut file = File::create(loc)?;
                                serialize_into(&mut file, val, Infinite)?;
                                Ok(())
                            });
                            persisted = Persisted::new(value, Arc::new(Mutex::new(persist)));
                        }
                        Ok(Arc::new(persisted))
                    }),
                ));
            }
            FileStore { dir, cache }
        }

        pub fn new_ref(&self, key: K) -> FileStored<K, V> {
            self.cache.new_ref(key)
        }

        pub fn insert(&self, key: K, value: V) -> Result<FileStored<K, V>, Error> {
            let mut loc = self.dir.clone();
            loc.push(&ToString::to_string(&key));
            let mut file = File::create(loc)?;
            serialize_into(&mut file, &value, Infinite)?;
            let res = self.cache.new_ref(key);
            Ok(res)
        }
    }
}
