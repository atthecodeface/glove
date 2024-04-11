//a Imports
use std::collections::HashMap;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;
use std::sync::{Mutex, MutexGuard};

use clap::Command;

use ic_base::Mesh;
use ic_cache::{Cache, CacheRef, Cacheable};
use ic_image::{Image, ImageGray16, ImageRgb8, Patch};
use ic_kernel::{KernelArgs, Kernels};
use ic_threads::ThreadPool;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum KeyType {
    ImagePath { path: PathBuf },
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
        }
    }
}
pub struct ImageCache {
    m_cache: Mutex<Cache<ImageCacheKey>>,
}
impl ImageCache {
    pub fn new() -> Self {
        let m_cache = Mutex::new(Cache::default());
        Self { m_cache }
    }

    pub fn src_image<P: AsRef<Path>>(&mut self, path: P) -> Result<CacheRef, String> {
        let mut cache = self.m_cache.lock().map_err(|e| format!("{e:?}"))?;
        let key = ImageCacheKey::of_image_path(&path);
        if !cache.contains(&key) {
            let src_img = ImageRgb8::read_image(path)?;
            let src_img = ImageCacheEntry::Rgb(src_img);
            cache.insert(key.clone(), src_img);
        }
        Ok(cache.get(&key).unwrap())
    }
}
