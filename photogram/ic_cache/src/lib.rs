//a Imports
use std::any::Any;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

//a Cacheable
pub trait Cacheable<Key>: Any
where
    Key: Hash + Ord + Clone + Sized + Eq + 'static,
{
    fn key(&self) -> &Key;
    // fn as_any(&self) -> &(dyn Any + '_);
    fn as_any(&self) -> &dyn Any;
    fn size(&self) -> usize;
}

pub trait Blah {
    //mp downcast
    fn downcast<T: 'static>(&self) -> Option<&T>;
}
impl<Key> Blah for Rc<dyn Cacheable<Key>>
where
    Key: Hash + Ord + Clone + Sized + Eq + 'static,
{
    fn downcast<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}
//a CacheEntry
//tp CacheEntry
struct CacheEntry<Key>
where
    Key: Hash + Ord + Clone + Sized + Eq + 'static,
{
    key: Key,
    data: Option<Rc<dyn Cacheable<Key>>>,
    last_use: usize,
    size: usize,
}

//ip CacheEntry
impl<Key> CacheEntry<Key>
where
    Key: Hash + Ord + Clone + Sized + Eq + 'static,
{
    //cp new
    fn new(e: &Rc<dyn Cacheable<Key>>, use_time: usize) -> Self {
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

    //mp is_empty
    fn is_empty(&self) -> bool {
        self.data.is_none()
    }

    //mp key
    fn key(&self) -> &Key {
        &self.key
    }

    //mp use_at
    fn use_at(&mut self, use_time: usize) {
        self.last_use = use_time;
    }

    //mp can_empty
    fn can_empty(&self) -> bool {
        if let Some(rc_e) = self.data.as_ref() {
            std::rc::Rc::strong_count(rc_e) == 1
        } else {
            false
        }
    }

    //mp take_copy
    fn take_copy(&mut self, use_time: usize) -> Option<Rc<dyn Cacheable<Key>>> {
        if let Some(rc_e) = self.data.as_ref() {
            self.last_use = use_time;
            Some(rc_e.clone())
        } else {
            None
        }
    }

    //mp fill
    fn fill(&mut self, e: &Rc<dyn Cacheable<Key>>, use_time: usize) -> bool {
        if self.is_empty() {
            self.data = Some(e.clone());
            self.last_use = use_time;
            true
        } else {
            false
        }
    }
}

//a Cache
//tp Cache
pub struct Cache<Key>
where
    Key: Hash + Ord + Clone + Sized + Eq + 'static,
{
    use_count: usize,
    total_size: usize,
    entries: Vec<CacheEntry<Key>>,
    index: HashMap<Key, usize>,
}

//ip Default for Cache
impl<Key> std::default::Default for Cache<Key>
where
    Key: Hash + Ord + Clone + Sized + Eq + 'static,
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
    Key: Hash + Ord + Clone + Sized + Eq + 'static,
{
    //mp contains
    fn contains<Q>(&self, k: &Q) -> bool
    where
        Key: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(idx) = self.index.get(&k) {
            !self.entries[*idx].is_empty()
        } else {
            false
        }
    }

    //mp insert
    fn insert(&mut self, e: &Rc<dyn Cacheable<Key>>) -> bool {
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
    fn get<Q>(&mut self, k: &Q) -> Option<Rc<dyn Cacheable<Key>>>
    where
        Key: Borrow<Q>,
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
    impl crate::Cacheable<String> for CacheThing {
        fn key(&self) -> &String {
            &self.name
        }
        fn size(&self) -> usize {
            match self.thing {
                Thing::Int(_) => 8,
                Thing::Str(s) => 8 + s.len(),
            }
        }
        //        fn as_any(&self) -> &(dyn Any + '_) {
        fn as_any(&self) -> &(dyn Any) {
            self
        }
    }
}
#[test]
fn test_cache() -> Result<(), ()> {
    use test::{CacheThing, Thing};
    let mut cache = Cache::default();
    let x: Rc<dyn Cacheable<_>> = Rc::new(CacheThing::int("First", 0));
    assert!(cache.insert(&x), "Should be able to insert x");
    assert!(cache.contains("First"), "Cache must contain x");
    assert!(
        cache
            .get("First")
            .expect("Must contain x")
            .as_any()
            .downcast_ref::<CacheThing>()
            .expect("Must be a CacheThing")
            .thing
            == Thing::Int(0)
    );
    assert!(
        cache
            .get("First")
            .expect("Must contain x")
            .downcast::<CacheThing>()
            .expect("Must be a CacheThing")
            .thing
            == Thing::Int(0)
    );
    Ok(())
}
