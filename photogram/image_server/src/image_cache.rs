use std::path::Path;
use std::sync::Mutex;

use ic_photogram::CacheRef;
use ic_photogram::ImageDrawable;
use ic_photogram::Result;

/// A cache of images (Rgb8, Gray16, F32 array) that can be accessed by multiple
/// threads, through a mutex
#[derive(Debug)]
pub struct ImageCache {
    m_cache: Mutex<ic_photogram::ImageCache>,
}

impl Default for ImageCache {
    fn default() -> Self {
        let m_cache = Mutex::new(ic_photogram::ImageCache::default());
        Self { m_cache }
    }
}

impl ImageCache {
    /// Get a [CacheRef] for the given path
    ///
    /// This loads the image into the cache if it is not present (only RGB8 images can be loaded at present)
    pub fn src_image<P: AsRef<Path>>(&self, path: P) -> Result<CacheRef> {
        let mut cache = self.m_cache.lock().map_err(|e| format!("{e:?}"))?;
        cache.src_image(path.as_ref())
    }

    pub fn thumbnail<P: AsRef<Path>>(&self, path: P, size: (u32, u32)) -> Result<CacheRef> {
        let mut cache = self.m_cache.lock().map_err(|e| format!("{e:?}"))?;
        cache.thumbnail(path.as_ref(), size)
    }

    /// Shrink the cache to the desired size, returning the size it is after shrinking
    pub fn shrink_cache(&self, to_size: usize, max_entry_size: usize) -> Result<usize> {
        let mut cache = self.m_cache.lock().map_err(|e| format!("{e:?}"))?;
        Ok(cache.shrink_cache(to_size, max_entry_size))
    }
}
