//a Imports
use std::path::{Path, PathBuf};

use crate::{Image, ImageConvert, ImageLuma16, ImageLumaF32, ImageRgb8, ImageRgba8};
use ic_base::Result;
use ic_cache::{Cache, CacheRef, Cacheable};

/// The kind of a key into the image cache
///
/// Only PathBuf is used at present
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum KeyKind {
    ImagePath { path: PathBuf },
    Thumbnail { path: PathBuf, size: (u32, u32) },
    UserImage { name: String },
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
    /// Create an image cache key from a [Path]
    pub fn of_thumbnail<P: AsRef<Path>>(path: &P, size: (u32, u32)) -> Self {
        let key_kind = KeyKind::Thumbnail {
            path: path.as_ref().to_owned(),
            size,
        };
        Self { key_kind }
    }
    /// Create an image cache key from a [Path]
    pub fn of_user<I: Into<String>>(name: I) -> Self {
        let key_kind = KeyKind::UserImage { name: name.into() };
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
    Gray(ImageLuma16),
    /// An array of 'f32' of width*height
    LumaF32(ImageLumaF32),
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
                let (w, h) = i.dimensions();
                w as usize * h as usize * 4
            }
            ImageCacheEntry::Gray(i) => {
                let (w, h) = i.dimensions();
                w as usize * h as usize * 2
            }
            ImageCacheEntry::LumaF32(i) => {
                let (w, h) = i.dimensions();
                w as usize * h as usize * 4
            }
        }
    }
}
impl From<ImageRgb8> for ImageCacheEntry {
    fn from(value: ImageRgb8) -> Self {
        Self::Rgb(value)
    }
}

impl From<ImageLuma16> for ImageCacheEntry {
    fn from(value: ImageLuma16) -> Self {
        Self::Gray(value)
    }
}

impl From<ImageLumaF32> for ImageCacheEntry {
    fn from(value: ImageLumaF32) -> Self {
        Self::LumaF32(value)
    }
}

impl ImageConvert for ImageCacheEntry {
    fn as_luma_f32(&self, as_width: Option<u32>, scale: f32) -> ImageLumaF32 {
        match self {
            Self::Rgb(img) => img.as_luma_f32(as_width, scale),
            Self::LumaF32(img) => img.as_luma_f32(as_width, scale),
            Self::Gray(img) => img.as_luma_f32(as_width, scale),
        }
    }
    fn as_rgb8(&self, as_width: Option<u32>, scale: f32) -> ImageRgb8 {
        match self {
            Self::Rgb(img) => img.as_rgb8(as_width, scale),
            Self::LumaF32(img) => img.as_rgb8(as_width, scale),
            Self::Gray(img) => img.as_rgb8(as_width, scale),
        }
    }

    fn as_rgba8(&self, as_width: Option<u32>, scale: f32) -> ImageRgba8 {
        match self {
            Self::Rgb(img) => img.as_rgba8(as_width, scale),
            Self::LumaF32(img) => img.as_rgba8(as_width, scale),
            Self::Gray(img) => img.as_rgba8(as_width, scale),
        }
    }

    fn as_luma16(&self, as_width: Option<u32>, scale: f32) -> ImageLuma16 {
        match self {
            Self::Rgb(img) => img.as_luma16(as_width, scale),
            Self::LumaF32(img) => img.as_luma16(as_width, scale),
            Self::Gray(img) => img.as_luma16(as_width, scale),
        }
    }
}

impl ImageCacheEntry {
    pub fn as_opt_rgb8(&self) -> Option<&ImageRgb8> {
        match &self {
            Self::Rgb(i) => Some(i),
            _ => None,
        }
    }
    pub fn as_opt_gray16(&self) -> Option<&ImageLuma16> {
        match &self {
            Self::Gray(i) => Some(i),
            _ => None,
        }
    }
    pub fn as_opt_f32(&self) -> Option<&ImageLumaF32> {
        match &self {
            Self::LumaF32(i) => Some(i),
            _ => None,
        }
    }

    /// Return an ImageRgb8 reference or panic if the entry is *not* one
    pub fn cr_as_rgb8(cr: &CacheRef) -> &ImageRgb8 {
        match cr.downcast::<Self>() {
            Some(Self::Rgb(i)) => i,
            _ => panic!("Is not right"),
        }
    }

    /// Return an ImageGray16 reference or panic if the entry is *not* one
    pub fn cr_as_gray(cr: &CacheRef) -> &ImageLuma16 {
        match cr.downcast::<Self>() {
            Some(Self::Gray(i)) => i,
            _ => panic!("Is not right"),
        }
    }

    /// Return a (width, height, data reference) tuple or panic if the entry is *not* an f32 array
    pub fn cr_as_f32(cr: &CacheRef) -> &ImageLumaF32 {
        match cr.downcast::<Self>() {
            Some(Self::LumaF32(i)) => i,
            _ => panic!("Is not right"),
        }
    }
}

/// A cache of images (Rgb8, Gray16, F32 array)
#[derive(Debug, Default)]
pub struct ImageCache {
    cache: Cache<ImageCacheKey>,
}

impl ImageCache {
    fn cache_src_image(&mut self, path: &Path) -> Result<CacheRef> {
        let key = ImageCacheKey::of_image_path(&path);
        if !self.cache.contains(&key) {
            eprintln!("Cache miss for {:?}", path);
            let src_img = ImageRgb8::read(path)?;
            let src_img = ImageCacheEntry::Rgb(src_img);
            self.cache.insert(key.clone(), src_img);
            eprintln!(
                "Cache now is {:.2} MB",
                (self.cache.total_size() as f64) / 1.0E6
            );
        }
        Ok(self.cache.get(&key).unwrap())
    }

    /// Add a user image to the cache and return its [CacheRef]; potentially
    /// drop a previous entry if replacing
    ///
    pub fn add_image<S: Into<String>, I: Into<ImageCacheEntry>>(
        &mut self,
        name: S,
        img: I,
        replace: bool,
    ) -> std::result::Result<CacheRef, I> {
        let name = name.into();
        let key = ImageCacheKey::of_user(name);
        if !self.cache.contains(&key) {
            self.cache.insert(key.clone(), img.into());
        } else if replace {
            self.cache.replace(key.clone(), img.into());
        } else {
            return Err(img);
        }
        Ok(self.cache.get(&key).unwrap())
    }

    /// Get a [CacheRef] for a user image
    pub fn get_image<S: Into<String>>(&mut self, name: S) -> Option<CacheRef> {
        let key = ImageCacheKey::of_user(name);
        self.cache.get(&key)
    }

    /// Get a [CacheRef] for the given path
    ///
    /// This loads the image into the cache if it is not present (only RGB8 images can be loaded at present)
    pub fn src_image<P: AsRef<Path>>(&mut self, path: P) -> Result<CacheRef> {
        self.cache_src_image(path.as_ref())
    }

    fn create_thumbnail(
        &mut self,
        key: &ImageCacheKey,
        path: &Path,
        size: (u32, u32),
    ) -> Result<()> {
        let src_img_ref = self.cache_src_image(&path)?;
        let src_img = ImageCacheEntry::cr_as_rgb8(&src_img_ref);

        let src_size = src_img.dimensions();
        let x_scale = (src_size.0 as f64) / (size.0 as f64);
        let y_scale = (src_size.1 as f64) / (size.1 as f64);
        let scale = x_scale.max(y_scale);
        let width = (src_size.0 as f64 / scale) as u32;
        let scaled_img = src_img.as_rgb8(Some(width), 1.0);
        let thumbnail_img = ImageCacheEntry::Rgb(scaled_img);
        self.cache.insert(key.clone(), thumbnail_img);
        Ok(())
    }

    pub fn thumbnail<P: AsRef<Path>>(&mut self, path: P, size: (u32, u32)) -> Result<CacheRef> {
        let key = ImageCacheKey::of_thumbnail(&path, size);
        if !self.cache.contains(&key) {
            eprintln!(
                "Cache miss for thumbnail {:?} {}x{}",
                path.as_ref(),
                size.0,
                size.1
            );
            self.create_thumbnail(&key, path.as_ref(), size)?;
            eprintln!(
                "Cache now is {:.2} MB",
                (self.cache.total_size() as f64) / 1.0E6
            );
        }
        Ok(self.cache.get(&key).unwrap())
    }

    /// Shrink the cache to the desired size, returning the size it is after shrinking
    pub fn shrink_cache(&mut self, to_size: usize, max_entry_size: usize) -> usize {
        self.cache
            .shrink_to(to_size, |c| c.size() >= max_entry_size);
        self.cache.total_size()
    }

    pub fn image_rgb8(cache_ref: &CacheRef) -> Option<&ImageRgb8> {
        let Some(ice) = cache_ref.downcast::<ImageCacheEntry>() else {
            return None;
        };
        ice.as_opt_rgb8()
    }
    pub fn image_gray16(cache_ref: &CacheRef) -> Option<&ImageLuma16> {
        let Some(ice) = cache_ref.downcast::<ImageCacheEntry>() else {
            return None;
        };
        ice.as_opt_gray16()
    }
    pub fn image_f32(cache_ref: &CacheRef) -> Option<&ImageLumaF32> {
        let Some(ice) = cache_ref.downcast::<ImageCacheEntry>() else {
            return None;
        };
        ice.as_opt_f32()
    }

    pub fn image_rgb8_err(cache_ref: &CacheRef) -> Result<&ImageRgb8> {
        Self::image_rgb8(cache_ref).ok_or_else(|| "Image required to be RGB8 but was not".into())
    }

    pub fn as_opt_luma_f32(
        cache_ref: &CacheRef,
        as_width: Option<u32>,
        scale: f32,
    ) -> Option<ImageLumaF32> {
        cache_ref
            .downcast::<ImageCacheEntry>()
            .map(|ice| ice.as_luma_f32(as_width, scale))
    }

    pub fn as_opt_rgb8(
        cache_ref: &CacheRef,
        as_width: Option<u32>,
        scale: f32,
    ) -> Option<ImageRgb8> {
        cache_ref
            .downcast::<ImageCacheEntry>()
            .map(|ice| ice.as_rgb8(as_width, scale))
    }

    pub fn as_opt_rgba8(
        cache_ref: &CacheRef,
        as_width: Option<u32>,
        scale: f32,
    ) -> Option<ImageRgba8> {
        cache_ref
            .downcast::<ImageCacheEntry>()
            .map(|ice| ice.as_rgba8(as_width, scale))
    }

    pub fn as_opt_luma16(
        cache_ref: &CacheRef,
        as_width: Option<u32>,
        scale: f32,
    ) -> Option<ImageLuma16> {
        cache_ref
            .downcast::<ImageCacheEntry>()
            .map(|ice| ice.as_luma16(as_width, scale))
    }
}
