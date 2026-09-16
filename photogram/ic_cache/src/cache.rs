//a Imports
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

use crate::{CacheEntry, CacheRef, Cacheable};

/// A cache of [CacheEntry] items, given a key
///
/// Each [CacheEntry] is an atomically reference-counted [Cacheable]; the
/// [Cacheable] can be downcast to its content type, and if it requires
/// mutability it must handle that using interior mutability. In a multithreaded
/// environment this should be with a Mutex or RwLock, for example.
#[derive(Debug)]
pub struct Cache<Key>
where
    Key: Hash + Ord + Sized + Eq + 'static,
{
    use_time: usize,
    total_size: usize,
    entries: Vec<CacheEntry>,
    index: HashMap<Key, usize>,
}

impl<Key> std::default::Default for Cache<Key>
where
    Key: Hash + Ord + Sized + Eq + 'static,
{
    fn default() -> Self {
        Cache {
            use_time: 0,
            total_size: 0,
            entries: vec![],
            index: HashMap::default(),
        }
    }
}

impl<Key> Cache<Key>
where
    Key: Hash + Ord + Sized + Eq + 'static,
{
    /// Get the total size of the cache
    pub fn total_size(&self) -> usize {
        self.total_size
    }

    /// Return true if the cache contains the key
    pub fn contains<Q>(&self, k: &Q) -> bool
    where
        Key: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(idx) = self.index.get(k) {
            !self.entries[*idx].is_empty()
        } else {
            false
        }
    }

    /// Insert an item into the cache given a key
    ///
    /// If there is an item in the cache already with that key, then an
    /// insertion is *NOT* performed and the value is returned
    pub fn insert<C: Cacheable>(&mut self, k: Key, e: C) -> Option<C> {
        if let Some(idx) = self.index.get(&k) {
            if self.entries[*idx].is_empty() {
                let size = e.size();
                self.entries[*idx].fill(e.into(), self.use_time);
                self.use_time += 1;
                self.total_size += size;
                None
            } else {
                Some(e)
            }
        } else {
            let size = e.size();
            let n = self.entries.len();
            self.entries.push(CacheEntry::new(e.into(), self.use_time));
            self.index.insert(k, n);
            self.use_time += 1;
            self.total_size += size;
            None
        }
    }

    /// Get a copy of the [CacheEntry] (as a [CacheRef]) of the item in the
    /// cache with the given key, or None if the key is not present or the cache
    /// entry associated with it is empty
    pub fn get<Q>(&mut self, k: &Q) -> Option<CacheRef>
    where
        Key: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(idx) = self.index.get(k) {
            let opt_e = self.entries[*idx].take_copy(self.use_time);
            self.use_time += 1;
            opt_e
        } else {
            None
        }
    }

    /// Generate an array of indices sorted by age
    pub fn indices_by_age<F>(&self, filter: F) -> Vec<usize>
    where
        F: Fn(&CacheEntry) -> bool,
    {
        let mut indices: Vec<usize> = vec![];
        for i in self.index.values() {
            if filter(&self.entries[*i]) {
                indices.push(*i);
            }
        }

        indices.sort_by(|a, b| {
            self.entries[*a]
                .last_use()
                .cmp(&self.entries[*b].last_use())
        });
        indices
    }

    /// Empty the entries, least recently used first, until the cache has a total size no more than required
    ///
    /// Returns true if the size has been reduced as desired; cache entries can
    /// only be emptied if they are not in use (i.e. the CacheEntry associated
    /// with them has not got an active clone)
    pub fn shrink_to<F>(&mut self, size: usize, filter: F) -> bool
    where
        F: Fn(&CacheEntry) -> bool,
    {
        eprintln!("Shrink to {size} when at {}", self.total_size);
        if self.total_size < size {
            return true;
        }
        let indices = self.indices_by_age(filter);
        eprintln!("indices {indices:?}");
        for i in indices.into_iter() {
            eprintln!("Index {i}, {}", self.total_size);
            if self.total_size < size {
                return true;
            }
            self.total_size -= self.entries[i].empty();
        }
        self.total_size < size
    }
}
