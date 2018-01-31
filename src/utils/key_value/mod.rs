pub trait KeyValueStore<'a, K, V> {
    type Error;
    type Entry: Entry<V>;
    /// Must only succeed if there was previously no value associated with `key`.
    fn insert(&'a mut self, key: K, value: V) -> Result<Self::Entry, Self::Error>;
    fn lookup(&'a mut self, key: &'a K) -> Result<Self::Entry, Self::Error>;
    fn delete(&'a mut self, key: &'a K) -> Result<(), Self::Error>;
}

pub trait Entry<V> {
    type Error;
    fn read<T, F: FnOnce(&V) -> T>(&self, fun: F) -> Result<T, Self::Error>;
    fn write<T, F: FnOnce(&mut V) -> T>(&mut self, fun: F) -> Result<T, Self::Error>;
}

pub mod cache;
pub mod singleton;

pub mod in_memory;
