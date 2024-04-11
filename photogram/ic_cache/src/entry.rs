//a Imports
use std::hash::Hash;
use std::rc::Rc;

use crate::Cacheable;

pub struct CacheRef {
    data: Rc<dyn Cacheable>,
}
impl CacheRef {
    fn ref_cnt(&self) -> usize {
        std::rc::Rc::strong_count(&self.data)
    }
    #[inline]
    pub fn new<C: Cacheable>(c: C) -> Self {
        let data: Rc<dyn Cacheable> = Rc::new(c);
        Self { data }
    }
    pub fn downcast<'a, T: 'static>(&'a self) -> Option<&'a T> {
        self.data.as_any().downcast_ref::<T>()
    }
}

impl<C: Cacheable> From<C> for CacheRef {
    fn from(c: C) -> Self {
        Self::new(c)
    }
}

impl std::clone::Clone for CacheRef {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}
impl std::ops::Deref for CacheRef {
    type Target = Rc<dyn Cacheable>;
    fn deref(&self) -> &Rc<dyn Cacheable> {
        &self.data
    }
}

impl std::convert::AsRef<Rc<dyn Cacheable>> for CacheRef {
    fn as_ref(&self) -> &Rc<dyn Cacheable> {
        &self.data
    }
}

//a CacheEntry
//tp CacheEntry
pub struct CacheEntry {
    data: Option<CacheRef>,
    last_use: usize,
    size: usize,
}

//ip CacheEntry
impl CacheEntry {
    //cp new
    pub fn new(e: CacheRef, use_time: usize) -> Self {
        let size = e.size();
        let data = Some(e);
        let last_use = use_time;
        Self {
            data,
            last_use,
            size,
        }
    }

    //ap last_use
    pub fn last_use(&self) -> usize {
        self.last_use
    }

    //mp is_empty
    pub fn is_empty(&self) -> bool {
        self.data.is_none()
    }

    //mp can_empty
    pub fn can_empty(&self) -> bool {
        if let Some(rc_e) = self.data.as_ref() {
            rc_e.ref_cnt() == 1
        } else {
            false
        }
    }

    //mp empty
    pub fn empty(&mut self) -> usize {
        if let Some(rc_e) = self.data.as_ref() {
            if rc_e.ref_cnt() == 1 {
                self.data = None;
                self.size
            } else {
                0
            }
        } else {
            0
        }
    }

    //mp take_copy
    pub fn take_copy(&mut self, use_time: usize) -> Option<CacheRef> {
        if let Some(rc_e) = self.data.as_ref() {
            self.last_use = use_time;
            Some(rc_e.clone())
        } else {
            None
        }
    }

    //mp fill
    pub fn fill(&mut self, e: CacheRef, use_time: usize) -> Option<CacheRef> {
        if self.is_empty() {
            self.data = Some(e);
            self.last_use = use_time;
            None
        } else {
            Some(e)
        }
    }
}
