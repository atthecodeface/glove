//a Imports
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::{CacheEntry, CacheRef, Cacheable};

//a Cache
//tp Cache
pub struct Cache<Key>
where
    Key: Hash + Ord + Sized + Eq + 'static,
{
    use_count: usize,
    total_size: usize,
    entries: Vec<CacheEntry>,
    index: HashMap<Key, usize>,
}

//ip Default for Cache
impl<Key> std::default::Default for Cache<Key>
where
    Key: Hash + Ord + Sized + Eq + 'static,
{
    fn default() -> Self {
        Cache {
            use_count: 0,
            total_size: 0,
            entries: vec![],
            index: HashMap::default(),
        }
    }
}

//ip Cache
impl<Key> Cache<Key>
where
    Key: Hash + Ord + Sized + Eq + 'static,
{
    //mp contains
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

    //mp insert
    pub fn insert<C: Cacheable>(&mut self, k: Key, e: C) -> Option<C> {
        if let Some(idx) = self.index.get(&k) {
            if self.entries[*idx].is_empty() {
                let size = e.size();
                self.entries[*idx].fill(e.into(), self.use_count);
                self.use_count += 1;
                self.total_size += size;
                None
            } else {
                Some(e)
            }
        } else {
            let size = e.size();
            let n = self.entries.len();
            self.entries.push(CacheEntry::new(e.into(), self.use_count));
            self.index.insert(k, n);
            self.use_count += 1;
            self.total_size += size;
            None
        }
    }

    //mp get
    pub fn get<Q>(&mut self, k: &Q) -> Option<CacheRef>
    where
        Key: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(idx) = self.index.get(k) {
            let opt_e = self.entries[*idx].take_copy(self.use_count);
            self.use_count += 1;
            opt_e
        } else {
            None
        }
    }

    //mp indices_by_age
    pub fn indices_by_age(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = self.index.values().copied().collect();
        indices.sort_by(|a, b| {
            self.entries[*a]
                .last_use()
                .cmp(&self.entries[*b].last_use())
        });
        indices
    }

    //mp shrink_to
    pub fn shrink_to(&mut self, size: usize) -> bool {
        eprintln!("Shrink to {size} when at {}", self.total_size);
        if self.total_size < size {
            return true;
        }
        let indices = self.indices_by_age();
        eprintln!("indices {:?}", indices);
        for i in indices.into_iter() {
            eprintln!("Index {i}, {}", self.total_size);
            if self.total_size < size {
                return true;
            }
            self.total_size -= self.entries[i].empty();
        }
        self.total_size < size
    }

    //zz All done
}
