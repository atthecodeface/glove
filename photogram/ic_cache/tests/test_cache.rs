//a Imports

use ic_cache::{Cache, Cacheable};

//a Cache
use std::any::Any;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
struct WrapUsize(usize);

impl WrapUsize {
    fn key(&self) -> String {
        self.0.to_string()
    }
}
//ip From<usize> for WrapUsize
impl From<usize> for WrapUsize {
    fn from(v: usize) -> Self {
        WrapUsize(v)
    }
}

//ip Cacheable for WrapUsize
impl ic_cache::Cacheable for WrapUsize {
    fn size(&self) -> usize {
        4
    }
    fn as_any(&self) -> &(dyn Any) {
        self
    }
}

//tp Thing
#[derive(Debug, PartialEq, Eq)]
pub enum Thing {
    Int(usize),
    Str(&'static str),
    Huge(usize),
}

//tp CacheThing
#[derive(Debug)]
pub struct CacheThing {
    pub thing: Thing,
}

//ip CacheThing
impl CacheThing {
    pub fn int(x: usize) -> Self {
        CacheThing {
            thing: Thing::Int(x),
        }
    }
    pub fn huge(s: usize) -> Self {
        CacheThing {
            thing: Thing::Huge(s),
        }
    }
}

//ip Cacheable for CacheThing
impl ic_cache::Cacheable for CacheThing {
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

//a Tests
//tp test_cache
#[test]
fn test_cache() -> Result<(), ()> {
    let mut cache = Cache::default();
    let x = CacheThing::int(0);
    let huge = CacheThing::huge(1000 * 1000);
    assert!(
        cache.insert("First".to_owned(), x).is_none(),
        "Should be able to insert x"
    );
    assert!(
        cache.insert("Huge 1".to_owned(), huge).is_none(),
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
    let wy: WrapUsize = 3_usize.into();
    let wk = wy.key();
    assert!(cache.insert(wk, wy).is_none());
    assert_eq!(
        cache.get("3").unwrap().downcast::<WrapUsize>().unwrap(),
        &3.into()
    );
    assert_eq!(
        cache
            .get("Huge 1")
            .unwrap()
            .downcast::<CacheThing>()
            .unwrap()
            .thing,
        Thing::Huge(1000 * 1000)
    );
    assert_eq!(
        cache.get("3").unwrap().downcast::<WrapUsize>().unwrap(),
        &3.into()
    );
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
