use std::{collections::HashMap, default};

use ic_base::{Result, Rrc};
use ic_image::{Image, ImageDrawable, ImageRgb8, ImageSquareSet, ImageSquares};
use ic_mapping::NamedPoint;

#[derive(Debug)]
pub struct CipImages<I: Clone + ImageDrawable> {
    images: HashMap<usize, I>,
}

impl<I: Clone + ImageDrawable> std::default::Default for CipImages<I> {
    fn default() -> Self {
        let images = HashMap::default();
        Self { images }
    }
}

impl<I: Clone + ImageDrawable> CipImages<I> {
    fn find_cip(&self, cip_index: usize) -> Option<I> {
        self.images.get(&cip_index).cloned()
    }
    fn find_or_add_cip<F: FnOnce() -> Option<I>>(
        &mut self,
        cip_index: usize,
        width: u32,
        height: u32,
        image_fn: F,
    ) -> Option<I> {
        if let Some(image) = self.images.get(&cip_index).cloned() {
            if image.size() == (width, height) {
                return Some(image);
            }
        }
        let _ = self.images.remove(&cip_index);
        if let Some(image) = image_fn() {
            self.images.insert(cip_index, image.clone());
            Some(image)
        } else {
            None
        }
    }
}
#[derive(Debug)]
pub struct NamedPointImages {
    /// The set of square regions in the images for each named point
    ///
    /// This contains an Rrc of the underlying image
    image_squares: ImageSquareSet<ImageRgb8>,
    /// Map from named point name to arrays of  (CIP number  ImageSquares which includes a reference to the underyling image)
    ///
    /// This is treated somewhat is a cache - when the CIPs are changed, or named points are adjusted.
    np_cip_images: HashMap<String, CipImages<ImageSquares<ImageRgb8>>>,
}

impl std::default::Default for NamedPointImages {
    /// Default is required for a Project, but the image is not of much use
    fn default() -> Self {
        let image = ImageRgb8::new(256, 256);
        Self::create(image, 8).unwrap()
    }
}

impl NamedPointImages {
    pub fn create(image: ImageRgb8, square_size: u32) -> Result<Self> {
        let image_squares = ImageSquareSet::create(image, square_size)?;
        let np_cip_images = HashMap::default();
        Ok(Self {
            image_squares,
            np_cip_images,
        })
    }

    pub fn clear(&mut self) {
        self.np_cip_images.clear();
    }

    pub fn reset(&mut self, image: ImageRgb8, square_size: u32) -> Result<()> {
        self.clear();
        self.image_squares = ImageSquareSet::create(image, square_size)?;
        Ok(())
    }

    pub fn np_clear(&mut self, np: NamedPoint) {
        let np_tag = np.ref_tag();
        let np = np_tag.as_str();
        self.np_cip_images.remove(np);
    }

    pub fn np_find_cip(&self, np: NamedPoint, cip_idx: usize) -> Option<ImageSquares<ImageRgb8>> {
        let np_tag = np.ref_tag();
        let np = np_tag.as_str();
        let Some(np_cips) = self.np_cip_images.get(np) else {
            return None;
        };
        np_cips.find_cip(cip_idx)
    }

    /// Add an image rectangle for a CIP to the named point
    ///
    /// If there is already an image rectangle for that CIP for that named point
    /// of the specified size, then return that, else create one if possible
    pub fn np_find_or_add_cip(
        &mut self,
        np: NamedPoint,
        cip_idx: usize,
        width: u32,
        height: u32,
    ) -> Option<ImageSquares<ImageRgb8>> {
        let np_tag = np.ref_tag();
        let np = np_tag.as_str();
        let np_cip_images = &mut self.np_cip_images;
        let image_squares = &mut self.image_squares;
        let sq_size = image_squares.square_size();
        let width = width.div_ceil(sq_size) * sq_size;
        let height = height.div_ceil(sq_size) * sq_size;
        let np_cips = np_cip_images
            .entry(np.to_owned())
            .or_insert_with(|| CipImages::default());
        np_cips.find_or_add_cip(cip_idx, width, height, || {
            image_squares.allocate_squares(width, height)
        })
    }

    pub fn image(&self) -> &Rrc<ImageRgb8> {
        self.image_squares.image()
    }

    pub fn np_cip_fill(&self, )
}
