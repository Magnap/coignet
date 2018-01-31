use super::KeyValueStore;
use std::sync::{Arc, RwLock, atomic::{AtomicBool, Ordering}};
use std::collections::HashMap;
use std::hash::Hash;

type Pool<K, E> = HashMap<K, Entry<K, E>>;

pub struct Singleton<K: Hash + Eq, E, KVS> {
    pub store: KVS,
    pool: Arc<RwLock<Pool<K, E>>>,
}

pub struct Entry<K: Hash + Eq, E> {
    ptr: Arc<SingletonEntry<K, E>>,
}

impl<K: Hash + Eq, E> Clone for Entry<K, E> {
    fn clone(&self) -> Self {
        Entry {
            ptr: self.ptr.clone(),
        }
    }
}

struct SingletonEntry<K: Hash + Eq, E> {
    pool: Arc<RwLock<Pool<K, E>>>,
    deleted: AtomicBool,
    entry: RwLock<E>,
    key: K,
}

impl<K: Hash + Eq, E> Drop for SingletonEntry<K, E> {
    fn drop(&mut self) {
        let mut pool = self.pool.write().expect("singleton pool lock poisoned");
        pool.remove(&self.key);
    }
}

pub enum Error<E> {
    SingletonDeleted,
    Other(E),
}

impl<K: Hash + Eq, V, E: super::Entry<V>> super::Entry<V> for Entry<K, E> {
    type Error = Error<<E as super::Entry<V>>::Error>;

    fn read<T, F: FnOnce(&V) -> T>(&self, fun: F) -> Result<T, Self::Error> {
        if self.ptr.deleted.load(Ordering::SeqCst) {
            return Err(Error::SingletonDeleted);
        }
        let entry = self.ptr
            .entry
            .read()
            .expect("singleton entry lock poisoned");
        entry.read(fun).map_err(Error::Other)
    }
    fn write<T, F: FnOnce(&mut V) -> T>(&mut self, fun: F) -> Result<T, Self::Error> {
        if self.ptr.deleted.load(Ordering::SeqCst) {
            return Err(Error::SingletonDeleted);
        }
        let mut entry = self.ptr
            .entry
            .write()
            .expect("singleton entry lock poisoned");
        entry.write(fun).map_err(Error::Other)
    }
}

impl<'a, K: Eq + Hash + Clone, V, KVS: KeyValueStore<'a, K, V>> KeyValueStore<'a, K, V>
    for Singleton<K, KVS::Entry, KVS>
{
    type Error = KVS::Error;
    type Entry = Entry<K, KVS::Entry>;

    fn insert(&'a mut self, key: K, value: V) -> Result<Self::Entry, Self::Error> {
        let mut pool = self.pool.write().expect("singleton pool lock poisoned");
        let raw_entry = self.store.insert(key.clone(), value)?;
        let entry = Entry {
            ptr: Arc::new(SingletonEntry {
                key: key.clone(),
                entry: RwLock::new(raw_entry),
                pool: self.pool.clone(),
                deleted: false.into(),
            }),
        };
        pool.insert(key, entry.clone());
        Ok(entry)
    }
    fn lookup(&'a mut self, key: &'a K) -> Result<Self::Entry, Self::Error> {
        let mut pool = self.pool.write().expect("singleton pool lock poisoned");
        {
            if let Some(entry) = pool.get(key) {
                return Ok(entry.clone());
            }
        }
        let raw_entry = self.store.lookup(key)?;
        let entry = Entry {
            ptr: Arc::new(SingletonEntry {
                key: key.clone(),
                entry: RwLock::new(raw_entry),
                pool: self.pool.clone(),
                deleted: false.into(),
            }),
        };
        pool.insert(key.clone(), entry.clone());
        Ok(entry)
    }
    fn delete(&'a mut self, key: &'a K) -> Result<(), Self::Error> {
        let pool = self.pool.read().expect("singleton pool lock poisoned");
        if let Some(entry) = pool.get(key) {
            entry.ptr.deleted.store(true, Ordering::SeqCst);
        }
        self.store.delete(key)
    }
}
