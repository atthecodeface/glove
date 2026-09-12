//a Imports
use std::sync::Arc;

use crate::Cacheable;

/// A reference to a [Cacheable], held as an atomic reference-counted object
///
/// If the [Cacheable] is to be mutable it requires *interior* mutability
///
/// This *does not* support Clone, as the elements have to be managed correctly
/// by the CacheEntry - it has to know how many copies there are, so clone is
/// implemented through the CacheEntry structure
pub struct CacheRef {
    data: Arc<dyn Cacheable>,
}

impl std::fmt::Debug for CacheRef {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "CacheRef({:?} of {:?})",
            Arc::as_ptr(&self.data),
            self.data.as_any().type_id()
        )
    }
}

impl CacheRef {
    /// Get the reference count,
    fn ref_cnt(&self) -> usize {
        Arc::strong_count(&self.data)
    }

    /// Create a new CacheRef, by consuming the [Cacheable]
    #[inline]
    pub fn new<C: Cacheable>(cache_entry: C) -> Self {
        let data: Arc<dyn Cacheable> = Arc::new(cache_entry);
        Self { data }
    }

    /// Retrieve (if the correct type) a reference to the [Cacheable]
    pub fn downcast<T: 'static>(&self) -> Option<&T> {
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
    type Target = Arc<dyn Cacheable>;
    fn deref(&self) -> &Arc<dyn Cacheable> {
        &self.data
    }
}

impl std::convert::AsRef<Arc<dyn Cacheable>> for CacheRef {
    fn as_ref(&self) -> &Arc<dyn Cacheable> {
        &self.data
    }
}

/// An entry in a Cache, containing a given [Cacheable] instance
#[derive(Debug)]
pub struct CacheEntry {
    data: Option<CacheRef>,
    last_use: usize,
    size: usize,
}

impl CacheEntry {
    /// Create a new [CacheEntry] for a given [CacheRef]
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

    /// Get the size of the cache entry
    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get the last use time of the cache entry
    pub fn last_use(&self) -> usize {
        self.last_use
    }

    /// Return true if the [CacheEntry] *is* empty
    pub fn is_empty(&self) -> bool {
        self.data.is_none()
    }

    /// Return true if the [CacheEntry] can be emptied; it cannot be emptied if
    /// there are any outstanding cloned references
    #[allow(dead_code)]
    pub fn can_empty(&self) -> bool {
        if let Some(rc_e) = self.data.as_ref() {
            rc_e.ref_cnt() == 1
        } else {
            false
        }
    }

    /// Empty the [CacheRef], and return the size of data freed-up
    ///
    /// This leaves the [CacheEntry] empty *if* it has no outstanding clones;
    /// otherwise it has no effect
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

    /// Take a copy (clone) of the entry, updating its last use time
    pub fn take_copy(&mut self, use_time: usize) -> Option<CacheRef> {
        if let Some(rc_e) = self.data.as_ref() {
            self.last_use = use_time;
            Some(rc_e.clone())
        } else {
            None
        }
    }

    /// Fill the [CacheEntry] *IF IT IS EMPTY*, updating the last use time
    ///
    /// If it is *not* empty then return the element that was attempting to fill
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
