use ic_base::{JsonParsable, PathSet};
use ic_image::Image;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::SphericalImageError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageFileDesc {
    path: PathBuf,
    img_wh: (u32, u32),
}

impl JsonParsable for ImageFileDesc {
    type PostParseArg = ();
    type PostParseResult = Self;
    fn reason() -> &'static str {
        "SphericalImageFile"
    }
    fn post_parse(self, _args: &()) -> ic_base::Result<Self> {
        Ok(self)
    }
}

#[derive(Debug)]
pub struct ImageFile<I: Image> {
    path: PathBuf,
    img_wh: (u32, u32),
    image: I,
}

impl<I: Image> ImageFile<I> {
    pub fn new(width: u32, height: u32) -> Self {
        let image = I::new(width, height);
        Self {
            path: PathBuf::new(),
            img_wh: (width, height),
            image,
        }
    }

    pub fn of_desc(path_set: &PathSet, desc: &ImageFileDesc) -> ic_base::Result<Self> {
        let p = path_set.find_file_err(&desc.path)?;
        let image = I::read(&p)?;
        let wh = image.size();
        if wh != desc.img_wh {
            return Err(SphericalImageError::BadImageFileSize(wh, desc.img_wh).into());
        }
        // if image_wh !+ desc.wh error
        Ok(Self {
            path: desc.path.clone(),
            img_wh: desc.img_wh,
            image,
        })
    }

    pub fn of_file<A: AsRef<Path>>(
        path_set: &PathSet,
        path: A,
        img_wh: Option<(u32, u32)>,
    ) -> ic_base::Result<Self> {
        let path = path.as_ref().to_owned();
        let filename = path_set.find_file_err(&path)?;
        let image = I::read_or_create_image(Some(filename), img_wh)?;
        let img_wh = image.size();
        Ok(Self {
            path,
            img_wh,
            image,
        })
    }

    pub fn to_desc(&self) -> ImageFileDesc {
        let path = self.path.clone();
        let img_wh = self.img_wh;
        ImageFileDesc { path, img_wh }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn set_path<P: AsRef<Path>>(&mut self, path: P) {
        self.path = path.as_ref().to_owned();
    }

    pub fn image(&self) -> &I {
        &self.image
    }

    pub fn image_mut(&mut self) -> &mut I {
        &mut self.image
    }
}
