//a Imports
use std::any::Any;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

//a Cacheable
/// Note that Any requires 'static, so we require that here too
pub trait Cacheable<Key>: Any + 'static
where
    Key: Hash + Ord + Clone + Sized + Eq + 'static,
{
    fn key(&self) -> Key;
    // fn as_any(&self) -> &(dyn Any + '_);
    fn as_any(&self) -> &dyn Any;
    fn size(&self) -> usize;
}

//tp CacheAccess
/// This trait is implemented for Rc<dyn Cacheable> so
/// that the content can be easily accessed
pub trait CacheAccess {
    //mp downcast
    fn downcast<T: 'static>(&self) -> Option<&T>;
}

//ip CacheAccess for Rc<dyn Cacheable<Key>>
impl<Key> CacheAccess for Rc<dyn Cacheable<Key>>
where
    Key: Hash + Ord + Clone + Sized + Eq + 'static,
{
    fn downcast<'a, T: 'static>(&'a self) -> Option<&'a T> {
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
    fn new(e: Rc<dyn Cacheable<Key>>, use_time: usize) -> Self {
        let key = e.key().clone();
        let size = e.size();
        let data = Some(e);
        let last_use = use_time;
        Self {
            key,
            data,
            last_use,
            size,
        }
    }

    //ap last_use
    fn last_use(&self) -> usize {
        self.last_use
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

    //mp empty
    fn empty(&mut self) -> usize {
        eprintln!("Try to empty {}", self.size);
        if let Some(rc_e) = self.data.as_ref() {
            eprintln!("Is some count {}", std::rc::Rc::strong_count(rc_e));
            if std::rc::Rc::strong_count(rc_e) == 1 {
                self.data = None;
                eprintln!("Emptyied size {}", self.size);
                self.size
            } else {
                0
            }
        } else {
            0
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
    fn fill(
        &mut self,
        e: Rc<dyn Cacheable<Key>>,
        use_time: usize,
    ) -> Option<Rc<dyn Cacheable<Key>>> {
        if self.is_empty() {
            self.data = Some(e);
            self.last_use = use_time;
            None
        } else {
            Some(e)
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
    fn insert(&mut self, e: Rc<dyn Cacheable<Key>>) -> Option<Rc<dyn Cacheable<Key>>> {
        let k = e.key();
        if let Some(idx) = self.index.get(&k) {
            if self.entries[*idx].is_empty() {
                let size = e.size();
                self.entries[*idx].fill(e, self.use_count);
                self.use_count += 1;
                self.total_size += size;
                None
            } else {
                Some(e)
            }
        } else {
            let size = e.size();
            let n = self.entries.len();
            self.entries.push(CacheEntry::new(e, self.use_count));
            self.index.insert(k.clone(), n);
            self.use_count += 1;
            self.total_size += size;
            None
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

    //mp indices_by_age
    fn indices_by_age(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = self.index.values().map(|x| *x).collect();
        indices.sort_by(|a, b| {
            self.entries[*a]
                .last_use()
                .cmp(&self.entries[*b].last_use())
        });
        indices
    }

    //mp shrink_to
    fn shrink_to(&mut self, size: usize) -> bool {
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

//a Tests
//mi Mod test
#[cfg(test)]
mod test {
    use std::any::Any;
    impl crate::Cacheable<String> for usize {
        fn key(&self) -> String {
            self.to_string()
        }
        fn size(&self) -> usize {
            4
        }
        fn as_any(&self) -> &(dyn Any) {
            self
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    pub(crate) enum Thing {
        Int(usize),
        Str(&'static str),
        Huge(usize),
    }
    #[derive(Debug)]
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
        pub fn huge(name: &str, s: usize) -> Self {
            CacheThing {
                name: name.into(),
                thing: Thing::Huge(s),
            }
        }
    }
    impl crate::Cacheable<String> for CacheThing {
        fn key(&self) -> String {
            self.name.clone()
        }
        fn size(&self) -> usize {
            match self.thing {
                Thing::Int(_) => 8,
                Thing::Str(s) => 8 + s.len(),
                Thing::Huge(s) => s,
            }
        }
        fn as_any(&self) -> &(dyn Any) {
            self
        }
    }
}

//tp test_cache
#[test]
fn test_cache() -> Result<(), ()> {
    use test::{CacheThing, Thing};
    let mut cache = Cache::default();
    let x: Rc<dyn Cacheable<_>> = Rc::new(CacheThing::int("First", 0));
    let huge: Rc<dyn Cacheable<_>> = Rc::new(CacheThing::huge("Huge 1", 1000 * 1000));
    assert!(cache.insert(x).is_none(), "Should be able to insert x");
    assert!(
        cache.insert(huge).is_none(),
        "Should be able to insert huge"
    );
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
    let y: Rc<dyn Cacheable<_>> = Rc::new(3_usize);
    assert!(cache.insert(y).is_none());
    assert_eq!(cache.get("3").unwrap().downcast::<usize>().unwrap(), &3);
    assert_eq!(
        cache
            .get("Huge 1")
            .unwrap()
            .downcast::<CacheThing>()
            .unwrap()
            .thing,
        Thing::Huge(1000 * 1000)
    );
    assert_eq!(cache.get("3").unwrap().downcast::<usize>().unwrap(), &3);
    cache.shrink_to(10_000_000);
    assert!(cache.contains("Huge 1"), "Should still contain Huge 1");
    assert!(
        cache.contains("First"),
        "Should still contain Huge 1, First and 3"
    );
    assert!(
        cache.contains("3"),
        "Should still contain Huge 1, First and 3"
    );
    cache.shrink_to(1_000_000);
    assert!(
        !cache.contains("Huge 1"),
        "Should not contain Huge 1, nor First"
    );
    assert!(!cache.contains("First"), "Should not contain First");
    assert!(cache.contains("3"), "Should still contain 3");
    cache.shrink_to(0);
    assert!(!cache.contains("Huge 1"), "Should not contain Huge 1");
    assert!(!cache.contains("First"), "Should not contain First");
    assert!(!cache.contains("3"), "Should not contain 3");
    Ok(())
}
