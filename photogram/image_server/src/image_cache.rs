//a Imports
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use ic_base::Result;
use ic_cache::{Cache, CacheRef, Cacheable};
use ic_image::{Image, ImageGray16, ImageRgb8};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum KeyType {
    ImagePath {
        path: PathBuf,
    },
    #[allow(dead_code)]
    Derived {
        name: String,
    },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct ImageCacheKey {
    key_type: KeyType,
}
impl ImageCacheKey {
    pub fn of_image_path<P: AsRef<Path>>(path: &P) -> Self {
        let key_type = KeyType::ImagePath {
            path: path.as_ref().to_owned(),
        };
        Self { key_type }
    }
}
#[derive(Debug)]
pub enum ImageCacheEntry {
    Rgb(ImageRgb8),
    #[allow(dead_code)]
    Gray(ImageGray16),
    #[allow(dead_code)]
    F32(usize, usize, Vec<f32>),
}
impl Cacheable for ImageCacheEntry {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
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
    fn as_rgb8(&self) -> &ImageRgb8 {
        match &self {
            Self::Rgb(i) => i,
            _ => panic!("Cannot unmap as ImageRgb8"),
        }
    }
    #[allow(dead_code)]
    fn as_gray16(&self) -> &ImageGray16 {
        match &self {
            Self::Gray(i) => i,
            _ => panic!("Cannot unmap as ImageGray16"),
        }
    }
    #[allow(dead_code)]
    fn as_f32(&self) -> (usize, usize, &[f32]) {
        match &self {
            Self::F32(w, h, v) => (*w, *h, v),
            _ => panic!("Cannot unmap as Float32 array"),
        }
    }
    pub fn cr_as_rgb8(cr: &CacheRef) -> &ImageRgb8 {
        cr.downcast::<Self>().unwrap().as_rgb8()
    }
    #[allow(dead_code)]
    pub fn cr_as_gray16(cr: &CacheRef) -> &ImageGray16 {
        cr.downcast::<Self>().unwrap().as_gray16()
    }
    #[allow(dead_code)]
    pub fn cr_as_f32(cr: &CacheRef) -> (usize, usize, &[f32]) {
        cr.downcast::<Self>().unwrap().as_f32()
    }
}

#[derive(Debug)]
pub struct ImageCache {
    m_cache: Mutex<Cache<ImageCacheKey>>,
}
impl ImageCache {
    pub fn new() -> Self {
        let m_cache = Mutex::new(Cache::default());
        Self { m_cache }
    }

    pub fn src_image<P: AsRef<Path>>(&self, path: P) -> Result<CacheRef> {
        let mut cache = self.m_cache.lock().map_err(|e| format!("{e:?}"))?;
        let key = ImageCacheKey::of_image_path(&path);
        if !cache.contains(&key) {
            eprintln!("Cache miss for {:?}", path.as_ref());
            let src_img = ImageRgb8::read_image(path)?;
            let src_img = ImageCacheEntry::Rgb(src_img);
            cache.insert(key.clone(), src_img);
        }
        Ok(cache.get(&key).unwrap())
    }

    #[allow(dead_code)]
    pub fn shrink_cache(&mut self, to_size: usize) -> Result<usize> {
        let mut cache = self.m_cache.lock().map_err(|e| format!("{e:?}"))?;
        cache.shrink_to(to_size);
        Ok(cache.total_size())
    }
}
