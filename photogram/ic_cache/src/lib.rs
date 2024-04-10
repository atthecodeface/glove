//a Imports
use std::any::Any;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

//a Cacheable
pub trait Cacheable: Any {
    type Key: Hash + Ord + Clone + Sized + Eq; // + 'static ?
    fn key(&self) -> &Self::Key;
    fn as_any(&self) -> &(dyn Any + '_);
    fn size(&self) -> usize;
}

struct CacheEntry<E: Cacheable> {
    key: E::Key,
    data: Option<Rc<E>>,
    last_use: usize,
    size: usize,
}

impl<E: Cacheable> CacheEntry<E> {
    fn new(e: &Rc<E>, use_time: usize) -> Self {
        let key = e.key().clone();
        let size = e.size();
        let data = Some(e.clone());
        let last_use = use_time;
        Self {
            key,
            data,
            last_use,
            size,
        }
    }
    fn is_empty(&self) -> bool {
        self.data.is_none()
    }
    fn key(&self) -> &E::Key {
        &self.key
    }
    fn use_at(&mut self, use_time: usize) {
        self.last_use = use_time;
    }
    fn can_empty(&self) -> bool {
        if let Some(rc_e) = self.data.as_ref() {
            std::rc::Rc::strong_count(rc_e) == 1
        } else {
            false
        }
    }
    fn take_copy(&mut self, use_time: usize) -> Option<Rc<E>> {
        if let Some(rc_e) = self.data.as_ref() {
            self.last_use = use_time;
            Some(rc_e.clone())
        } else {
            None
        }
    }
    fn fill(&mut self, e: &Rc<E>, use_time: usize) -> bool {
        if self.is_empty() {
            self.data = Some(e.clone());
            self.last_use = use_time;
            true
        } else {
            false
        }
    }
}

//tp Cache
pub struct Cache<E: Cacheable> {
    use_count: usize,
    total_size: usize,
    entries: Vec<CacheEntry<E>>,
    index: HashMap<E::Key, usize>,
}

//ip Default for Cache
impl<E: Cacheable> std::default::Default for Cache<E> {
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
impl<E: Cacheable> Cache<E> {
    //mp contains
    fn contains<Q>(&self, k: &Q) -> bool
    where
        E::Key: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(idx) = self.index.get(&k) {
            !self.entries[*idx].is_empty()
        } else {
            false
        }
    }

    //mp insert
    fn insert(&mut self, e: &Rc<E>) -> bool {
        let k = e.key();
        if let Some(idx) = self.index.get(&k) {
            if self.entries[*idx].is_empty() {
                let size = e.size();
                self.entries[*idx].fill(e, self.use_count);
                self.use_count += 1;
                self.total_size += size;
                true
            } else {
                false
            }
        } else {
            let size = e.size();
            let n = self.entries.len();
            self.entries.push(CacheEntry::new(e, self.use_count));
            self.index.insert(k.clone(), n);
            self.use_count += 1;
            self.total_size += size;
            true
        }
    }

    //mp get
    fn get<Q>(&mut self, k: &Q) -> Option<Rc<E>>
    where
        E::Key: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(idx) = self.index.get(&k) {
            let opt_e = self.entries[*idx].take_copy(self.use_count);
            self.use_count += 1;
            opt_e
        } else {
            None
        }
    }

    //zz All done
}

//a Tests
#[cfg(test)]
mod test {
    use std::any::Any;
    #[derive(PartialEq, Eq)]
    pub(crate) enum Thing {
        Int(usize),
        Str(&'static str),
    }
    pub(crate) struct CacheThing {
        pub name: String,
        pub thing: Thing,
    }
    impl CacheThing {
        pub fn int(name: &str, x: usize) -> Self {
            CacheThing {
                name: name.into(),
                thing: Thing::Int(x),
            }
        }
    }
    impl crate::Cacheable for CacheThing {
        type Key = String;
        fn key(&self) -> &String {
            &self.name
        }
        fn size(&self) -> usize {
            match self.thing {
                Thing::Int(_) => 8,
                Thing::Str(s) => 8 + s.len(),
            }
        }
        fn as_any(&self) -> &(dyn Any + '_) {
            self
        }
    }
}
#[test]
fn test_cache() -> Result<(), ()> {
    use test::{CacheThing, Thing};
    let mut cache = Cache::default();
    let x = Rc::new(CacheThing::int("First", 0));
    assert!(cache.insert(&x), "Should be able to insert x");
    assert!(cache.contains("First"), "Cache must contain x");
    assert!(cache.get("First").expect("Must contain x").thing == Thing::Int(0));
    Ok(())
}
