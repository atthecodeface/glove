//a Imports
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use photogram::Result;
use photogram::{Cache, CacheRef, Cacheable};
use photogram::{Image, ImageDrawable, ImageGray16, ImageRgb8};

/// The kind of a key into the image cache
///
/// Only PathBuf is used at present
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum KeyKind {
    ImagePath {
        path: PathBuf,
    },
    #[allow(dead_code)]
    Derived {
        name: String,
    },
}

/// A key into the image cache
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct ImageCacheKey {
    key_kind: KeyKind,
}

impl ImageCacheKey {
    /// Create an image cache key from a [Path]
    pub fn of_image_path<P: AsRef<Path>>(path: &P) -> Self {
        let key_kind = KeyKind::ImagePath {
            path: path.as_ref().to_owned(),
        };
        Self { key_kind }
    }
}

/// An entry in the ImageCache, which can be an actual image or an array of f32
/// of a given width*height
#[derive(Debug)]
pub enum ImageCacheEntry {
    /// An RGB8 image, usually from a JPEG
    Rgb(ImageRgb8),
    /// A gray-scale image
    Gray(ImageGray16),
    /// An array of 'f32' of width*height
    F32(usize, usize, Vec<f32>),
}

impl Cacheable for ImageCacheEntry {
    /// Cast the ImageCacheEntry to 'Any' for storage in the cache reference
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    /// Return the size of the cache entry from its content size
    fn size(&self) -> usize {
        match self {
            ImageCacheEntry::Rgb(i) => {
                let (w, h) = i.size();
                w as usize * h as usize * 4
            }
            ImageCacheEntry::Gray(i) => {
                let (w, h) = i.size();
                w as usize * h as usize * 2
            }
            ImageCacheEntry::F32(w, h, _) => w * h * 4,
        }
    }
}

impl ImageCacheEntry {
    /// Return an ImageRgb8 reference or panic if the entry is *not* one
    fn as_rgb8(&self) -> &ImageRgb8 {
        match &self {
            Self::Rgb(i) => i,
            _ => panic!("Cannot unmap as ImageRgb8"),
        }
    }

    /// Return an ImageGray16 reference or panic if the entry is *not* one
    fn as_gray16(&self) -> &ImageGray16 {
        match &self {
            Self::Gray(i) => i,
            _ => panic!("Cannot unmap as ImageGray16"),
        }
    }

    /// Return a (width, height, data reference) tuple or panic if the entry is *not* an f32 array
    fn as_f32(&self) -> (usize, usize, &[f32]) {
        match &self {
            Self::F32(w, h, v) => (*w, *h, v),
            _ => panic!("Cannot unmap as Float32 array"),
        }
    }

    /// Return an ImageRgb8 reference or panic if the entry is *not* one
    pub fn cr_as_rgb8(cr: &CacheRef) -> &ImageRgb8 {
        cr.downcast::<Self>().unwrap().as_rgb8()
    }

    /// Return an ImageGray16 reference or panic if the entry is *not* one
    pub fn cr_as_gray16(cr: &CacheRef) -> &ImageGray16 {
        cr.downcast::<Self>().unwrap().as_gray16()
    }

    /// Return a (width, height, data reference) tuple or panic if the entry is *not* an f32 array
    pub fn cr_as_f32(cr: &CacheRef) -> (usize, usize, &[f32]) {
        cr.downcast::<Self>().unwrap().as_f32()
    }
}

/// A cache of images (Rgb8, Gray16, F32 array) that can be accessed by multiple
/// threads, through a mutex
#[derive(Debug)]
pub struct ImageCache {
    m_cache: Mutex<Cache<ImageCacheKey>>,
}

impl Default for ImageCache {
    fn default() -> Self {
        let m_cache = Mutex::new(Cache::default());
        Self { m_cache }
    }
}

impl ImageCache {
    /// Get a [CacheRef] for the given path
    ///
    /// This loads the image into the cache if it is not present (only RGB8 images can be loaded at present)
    pub fn src_image<P: AsRef<Path>>(&self, path: P) -> Result<CacheRef> {
        let mut cache = self.m_cache.lock().map_err(|e| format!("{e:?}"))?;
        let key = ImageCacheKey::of_image_path(&path);
        if !cache.contains(&key) {
            eprintln!("Cache miss for {:?}", path.as_ref());
            // TDODO: Change to read
            let src_img = ImageRgb8::read(path)?;
            let src_img = ImageCacheEntry::Rgb(src_img);
            cache.insert(key.clone(), src_img);
            eprintln!("Cache now is {:.2} MB", (cache.total_size() as f64) / 1.0E6);
        }
        Ok(cache.get(&key).unwrap())
    }

    /// Shrink the cache to the desired size, returning the size it is after shrinking
    #[allow(dead_code)]
    pub fn shrink_cache(&mut self, to_size: usize) -> Result<usize> {
        let mut cache = self.m_cache.lock().map_err(|e| format!("{e:?}"))?;
        cache.shrink_to(to_size);
        Ok(cache.total_size())
    }
}
