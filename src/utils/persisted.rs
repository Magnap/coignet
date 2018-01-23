use failure::Error;
use std::ops::{Deref, DerefMut};
pub use std::sync::{Arc, Mutex};
use std::marker::PhantomData;

// TODO find a more elegant way to allow cloning of the persistor
type Persistor<T> = Arc<Mutex<Box<FnMut(&T) -> Result<(), Error>>>>;

pub struct Persisted<T, S: PersistenceStrategy = OnDrop> {
    data: T,
    persist: Persistor<T>,
    _strategy: PhantomData<S>,
}

impl<T, S: PersistenceStrategy> Persisted<T, S> {
    pub fn new(data: T, persist: Persistor<T>) -> Self {
        Persisted {
            _strategy: PhantomData,
            data,
            persist,
        }
    }
}

pub trait PersistenceStrategy {
    const ON_DROP: bool = false;
    const ON_READ: bool = false;
    const ON_WRITE: bool = false;
}

pub struct OnDrop;
impl PersistenceStrategy for OnDrop {
    const ON_DROP: bool = true;
}

pub struct OnRead;
impl PersistenceStrategy for OnRead {
    const ON_WRITE: bool = true;
    const ON_READ: bool = true;
}

pub struct OnWrite;
impl PersistenceStrategy for OnWrite {
    const ON_WRITE: bool = true;
}

pub struct ReadGuard<'a, T: 'a> {
    pub ptr: &'a T,
    persist: Option<Persistor<T>>,
}

impl<'a, T: 'a> Deref for ReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.ptr
    }
}

impl<'a, T: 'a> Drop for ReadGuard<'a, T> {
    fn drop(&mut self) {
        if let Some(ref persist) = self.persist {
            let mut persist = persist.lock().unwrap();
            (&mut *persist)(self.ptr).unwrap();
        }
    }
}

pub struct WriteGuard<'a, T: 'a> {
    pub ptr: &'a mut T,
    persist: Option<Persistor<T>>,
}

impl<'a, T: 'a> Deref for WriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.ptr
    }
}

impl<'a, T: 'a> DerefMut for WriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.ptr
    }
}

impl<'a, T: 'a> Drop for WriteGuard<'a, T> {
    fn drop(&mut self) {
        if let Some(ref persist) = self.persist {
            let mut persist = persist.lock().unwrap();
            (&mut *persist)(self.ptr).unwrap();
        }
    }
}

impl<T, S: PersistenceStrategy> Persisted<T, S> {
    pub fn read(&self) -> ReadGuard<T> {
        let persist = if S::ON_READ {
            Some(self.persist.clone())
        } else {
            None
        };
        ReadGuard {
            ptr: &self.data,
            persist,
        }
    }

    pub fn write(&mut self) -> WriteGuard<T> {
        let persist = if S::ON_WRITE {
            Some(self.persist.clone())
        } else {
            None
        };
        WriteGuard {
            ptr: &mut self.data,
            persist: persist,
        }
    }
}

impl<T, S: PersistenceStrategy> Drop for Persisted<T, S> {
    fn drop(&mut self) {
        if S::ON_DROP {
            let mut persist = self.persist.lock().unwrap();
            (&mut *persist)(&self.data).unwrap();
        }
    }
}
