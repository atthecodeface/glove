//! The concept of the ImageSquareSet is that it uses an Image of some form to contain sets
//! of square patches that can be copied to/from somewhere, that are identifiable bits of
//! an image
//!
//! Different squares from different images can then be compared for closeness
//!
//! The ImageSquareSet on a given image contents is essentially an arena of
//! squares which can be allocated and freed

use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use image::{DynamicImage, GenericImage, GenericImageView};

use ic_base::{Result, Rrc};

use crate::{Image, ImageDrawable};

/// The granularity of size for allocations
pub const TILE_SQUARE_SIZE: u32 = 8;

/// A set of image squares gathered from one or more images, with a backing store of 'I'
///
/// A subset of the image will be used, indicated by the used_squares
///
/// Each square in the image has the same width_sq and height_sq
#[derive(Debug)]
pub struct ImageSquareSet<I: Image> {
    image_filename: PathBuf,
    width_sq: u32,
    height_sq: u32,
    used_squares: Box<[u64]>,
    image: Rrc<I>,
}

//ip ImageSquareSet
impl<I> ImageSquareSet<I>
where
    I: Image,
{
    //ap image
    pub fn image(&self) -> &Rrc<I> {
        &self.image
    }

    //ap image_filename
    pub fn image_filename(&self) -> &Path {
        self.image_filename.as_path()
    }

    //mp set_image_filename
    pub fn set_image_filename<S: Into<PathBuf>>(&mut self, s: S) {
        self.image_filename = s.into();
    }

    /// Get the underlying tile
    pub fn square_size() -> u32 {
        TILE_SQUARE_SIZE
    }

    /// Create an [Self] from a given image, which must have a size that is a
    /// multiple (in each dimension) of the constant [TILE_SQUARE_SIZE]
    pub fn create(image: I) -> Result<Self> {
        let (w, h) = image.size();
        if !w.is_multiple_of(TILE_SQUARE_SIZE) || !h.is_multiple_of(TILE_SQUARE_SIZE) {
            return Err(
                format!("Image was not a multiple of the square size {TILE_SQUARE_SIZE}").into(),
            );
        }
        let width_sq = w / TILE_SQUARE_SIZE;
        let height_sq = h / TILE_SQUARE_SIZE;
        let used_squares_size = (width_sq * height_sq).div_ceil(64);
        let used_squares = vec![0_u64; used_squares_size as usize];
        let used_squares = used_squares.into_boxed_slice();
        let image_filename = PathBuf::new();
        let image = image.into();
        Ok(Self {
            image_filename,
            width_sq,
            height_sq,
            used_squares,
            image,
        })
    }

    /// Set the bit in the allocation array for tile (x,y)
    fn alloc_set_bit(&mut self, x: u32, y: u32) {
        let idx = (y * self.width_sq + x) as usize;
        let idx_bit = idx & 63;
        let idx = idx >> 6;
        self.used_squares[idx] |= 1 << (idx_bit as u64);
    }

    /// Clear the bit in the allocation array for tile (x,y)
    fn alloc_clr_bit(&mut self, x: u32, y: u32) {
        let idx = (y * self.width_sq + x) as usize;
        let idx_bit = idx & 63;
        let idx = idx >> 6;
        self.used_squares[idx] &= !(1 << (idx_bit as u64));
    }

    /// Get the allocation array bit (true for allocated, false for free) for tile (x,y)
    fn alloc_get_bit(&self, x: u32, y: u32) -> bool {
        let idx = (y * self.width_sq + x) as usize;
        let idx_bit = idx & 63;
        let idx = idx >> 6;
        (self.used_squares[idx] & (1 << (idx_bit as u64))) != 0
    }

    //mi find_first_free_square_from
    fn find_first_free_square_from(&self, mut i: u32) -> Option<u32> {
        while i < self.width_sq * self.height_sq {
            let idx = i as usize / 64;
            if i.is_multiple_of(64) {
                if self.used_squares[idx] != u64::MAX {
                    for j in 0..64 {
                        if (i + j) >= self.width_sq * self.height_sq {
                            return None;
                        }
                        if self.used_squares[idx] & (1 << j) == 0 {
                            return Some(i + j);
                        }
                    }
                }
            } else {
                let i_bit = i & 63;
                i -= i_bit;
                for j in i_bit..64 {
                    if (i + j) >= self.width_sq * self.height_sq {
                        return None;
                    }
                    if self.used_squares[idx] & (1 << j) == 0 {
                        return Some(i + j);
                    }
                }
            }
            i += 64;
        }
        None
    }

    /// Determine if the region starting at 'sq' and of the given width/height is all free in the allocation table
    fn is_region_free(&self, sq: u32, w_sq: u32, h_sq: u32) -> bool {
        let x_sq = sq % self.width_sq;
        let y_sq = sq / self.width_sq;
        if x_sq + w_sq <= self.width_sq && y_sq + h_sq <= self.height_sq {
            for y in 0..h_sq {
                for x in 0..w_sq {
                    if self.alloc_get_bit(x_sq + x, y_sq + y) {
                        return false;
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Find a free region, if possible, for a given width/height in square tiles
    fn find_free_region(&self, w_sq: u32, h_sq: u32) -> Option<(u32, u32)> {
        if w_sq * h_sq == 1 {
            self.find_first_free_square_from(0)
                .map(|sq| (sq % self.width_sq, sq / self.width_sq))
        } else {
            let mut i = 0;
            while let Some(sq) = self.find_first_free_square_from(i) {
                if self.is_region_free(sq, w_sq, h_sq) {
                    return Some((sq % self.width_sq, sq / self.width_sq));
                }
                i = sq + 1;
            }
            None
        }
    }

    /// Mark the region of tiles starting at (x_sq, y_sq) of given size (w_sq, h_sq) as allocated
    fn mark_alloc(&mut self, x_sq: u32, y_sq: u32, w_sq: u32, h_sq: u32) {
        for y in 0..h_sq {
            for x in 0..w_sq {
                self.alloc_set_bit(x_sq + x, y_sq + y);
            }
        }
    }

    /// Mark the region of tiles starting at (x_sq, y_sq) of given size (w_sq, h_sq) as free
    fn mark_free(&mut self, x_sq: u32, y_sq: u32, w_sq: u32, h_sq: u32) {
        for y in 0..h_sq {
            for x in 0..w_sq {
                self.alloc_clr_bit(x_sq + x, y_sq + y);
            }
        }
    }

    /// Allocate a region of
    #[track_caller]
    pub fn allocate_squares(&mut self, w: u32, h: u32) -> Option<ImageSquares<I>> {
        assert!(w.is_multiple_of(TILE_SQUARE_SIZE));
        assert!(h.is_multiple_of(TILE_SQUARE_SIZE));
        if let Some((x_sq, y_sq)) =
            self.find_free_region(w / TILE_SQUARE_SIZE, h / TILE_SQUARE_SIZE)
        {
            Some(self.select_squares(x_sq, y_sq, w, h, true))
        } else {
            None
        }
    }

    /// Mark the region of the [Self] denoted by sqs ([ImageSquares]) as free
    #[track_caller]
    pub fn free_squares(&mut self, sqs: ImageSquares<I>) {
        let (x_sq, y_sq, w, h) = sqs.take();
        self.mark_free(x_sq, y_sq, w / TILE_SQUARE_SIZE, h / TILE_SQUARE_SIZE)
    }

    /// Create an [ImageSquares] for a portion of the [ImageSquareSet], given a (x_sq,y_sq) tile and a width and height in tile squares
    ///
    /// Mark it as allocated if required; this is optional, as this method can
    /// be used to generate a sub-patch of a previous allocation; it can also be
    /// used to generate an initial allocation
    ///
    #[track_caller]
    pub fn select_squares(
        &mut self,
        x_sq: u32,
        y_sq: u32,
        w: u32,
        h: u32,
        mark_alloc: bool,
    ) -> ImageSquares<I> {
        assert!(w.is_multiple_of(TILE_SQUARE_SIZE));
        assert!(h.is_multiple_of(TILE_SQUARE_SIZE));
        if mark_alloc {
            self.mark_alloc(x_sq, y_sq, w / TILE_SQUARE_SIZE, h / TILE_SQUARE_SIZE);
        }
        ImageSquares::selected_squares(self, x_sq, y_sq, w, h)
    }
}

/// A square inside an image, given a width and a height
#[derive(Clone)]
pub struct ImageSquares<I: Image> {
    image: Rrc<I>,
    w: u32,
    h: u32,
    x_sq: u32,
    y_sq: u32,
}

impl<I> std::fmt::Debug for ImageSquares<I>
where
    I: Image,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "ImageSquares[{:?}:{}x{}@{},{}]",
            Rc::as_ptr(&self.image),
            self.w,
            self.h,
            self.x_sq * TILE_SQUARE_SIZE,
            self.y_sq * TILE_SQUARE_SIZE
        )
    }
}

impl<I> ImageSquares<I>
where
    I: Image,
{
    /// Get the starting square tile and size (in square tiles) of the [ImageSquares]
    pub fn take(self) -> (u32, u32, u32, u32) {
        (self.x_sq, self.y_sq, self.w, self.h)
    }

    /// Create an [ImageSquares] from an [ImageSquareSet] given
    #[track_caller]
    pub fn selected_squares(
        isqset: &ImageSquareSet<I>,
        x_sq: u32,
        y_sq: u32,
        w: u32,
        h: u32,
    ) -> Self {
        assert!(w.is_multiple_of(8));
        assert!(h.is_multiple_of(8));
        let image = isqset.image().clone();
        Self {
            image,
            x_sq,
            y_sq,
            w,
            h,
        }
    }

    /// Get the size in *pixels* of the square set
    pub fn size(&self) -> (u32, u32) {
        (self.w * TILE_SQUARE_SIZE, self.h * TILE_SQUARE_SIZE)
    }

    /// Copy data from an image at a starting (x,y) to this [ImageSquare]
    #[track_caller]
    pub fn copy_from_image(&self, image: &I, x: u32, y: u32)
    where
        I: std::ops::Deref<Target = DynamicImage>,
        I: std::ops::DerefMut,
    {
        self.image
            .borrow_mut()
            .copy_from(
                &*image.view(x, y, self.w, self.h),
                self.x_sq * TILE_SQUARE_SIZE,
                self.y_sq * TILE_SQUARE_SIZE,
            )
            .unwrap();
    }

    /// Copy data to an image at a starting (x,y) from this [ImageSquare]
    #[track_caller]
    pub fn copy_to_image(&self, image: &mut I, x: u32, y: u32)
    where
        I: std::ops::Deref<Target = DynamicImage>,
        I: std::ops::DerefMut,
    {
        image
            .copy_from(
                &*self.image.borrow().view(
                    self.x_sq * TILE_SQUARE_SIZE,
                    self.y_sq * TILE_SQUARE_SIZE,
                    self.w,
                    self.h,
                ),
                x,
                y,
            )
            .unwrap();
    }

    /// Copy data from one ImageSquare to another
    #[track_caller]
    pub fn copy_squares(&self, from: &Self)
    where
        I: std::ops::Deref<Target = DynamicImage>,
        I: std::ops::DerefMut,
    {
        assert_eq!(self.w, from.w);
        assert_eq!(self.h, from.h);
        if Rc::ptr_eq(&self.image, &from.image) {
            self.image.borrow_mut().copy_within(
                image::math::Rect {
                    x: from.x_sq * TILE_SQUARE_SIZE,
                    y: from.y_sq * TILE_SQUARE_SIZE,
                    width: self.w,
                    height: self.h,
                },
                self.x_sq * TILE_SQUARE_SIZE,
                self.y_sq * TILE_SQUARE_SIZE,
            );
        } else {
            self.copy_from_image(
                &from.image.borrow(),
                from.x_sq * TILE_SQUARE_SIZE,
                from.y_sq * TILE_SQUARE_SIZE,
            );
        }
    }
}

//ip ImageDrawable for ImageSquares
impl<I> ImageDrawable for ImageSquares<I>
where
    I: Image,
{
    type Pixel = I::Pixel;
    fn get(&self, x: u32, y: u32) -> Self::Pixel {
        self.image.borrow().get(
            x + (self.x_sq * TILE_SQUARE_SIZE),
            y + (self.y_sq * TILE_SQUARE_SIZE),
        )
    }
    fn put(&mut self, x: u32, y: u32, color: &Self::Pixel) {
        self.image.borrow_mut().put(
            x + (self.x_sq * TILE_SQUARE_SIZE),
            y + (self.y_sq * TILE_SQUARE_SIZE),
            color,
        )
    }
    fn blend(&mut self, x: u32, y: u32, blend: f64, color: &Self::Pixel) {
        self.image.borrow_mut().blend(
            x + (self.x_sq * TILE_SQUARE_SIZE),
            y + (self.y_sq * TILE_SQUARE_SIZE),
            blend,
            color,
        )
    }
    fn size(&self) -> (u32, u32) {
        (self.w, self.h)
    }
}

//a Tests
#[test]
fn test_image_square_0() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let image = crate::ImageRgb8::new(10 * 8, 20 * 8);
    let mut set = ImageSquareSet::create(image)?;
    for _ in 0..199 {
        let x = set.allocate_squares(8, 8);
        assert!(x.is_some());
        let x = x.unwrap();
        eprintln!("{x:?}");
    }
    let x = set.allocate_squares(16, 16);
    eprintln!("{x:?}");
    assert!(x.is_none());
    let x = set.allocate_squares(8, 8);
    eprintln!("{x:?}");
    assert!(x.is_some());
    let x = set.allocate_squares(8, 8);
    eprintln!("{x:?}");
    assert!(x.is_none());
    Ok(())
}

#[test]
fn test_image_square_1() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let image = crate::ImageRgb8::new(10 * 8, 20 * 8);
    let mut set = ImageSquareSet::create(image)?;
    for _ in 0..50 {
        let x = set.allocate_squares(16, 16);
        assert!(x.is_some());
        let x = x.unwrap();
        eprintln!("{x:?}");
    }
    let x = set.allocate_squares(16, 16);
    eprintln!("{x:?}");
    assert!(x.is_none());
    let x = set.allocate_squares(8, 8);
    eprintln!("{x:?}");
    assert!(x.is_none());
    Ok(())
}

#[test]
fn test_image_square_2() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let image = crate::ImageRgb8::new(10 * 8, 20 * 8);

    let mut set = ImageSquareSet::create(image)?;
    let first = set.allocate_squares(16, 16).unwrap();
    for _ in 0..49 {
        let _x = set.allocate_squares(16, 16);
    }
    assert!(set.allocate_squares(8, 8).is_none());
    set.free_squares(first);
    assert!(set.allocate_squares(8, 24).is_none());
    assert!(set.allocate_squares(24, 8).is_none());
    let x = set.allocate_squares(8, 16);
    assert!(x.is_some());
    let x = x.unwrap();
    assert!(set.allocate_squares(16, 8).is_none());
    let y = set.allocate_squares(8, 8).unwrap();
    let _z = set.allocate_squares(8, 8).unwrap();
    set.free_squares(x);
    assert!(set.allocate_squares(16, 8).is_none());
    set.free_squares(y);
    assert!(set.allocate_squares(16, 8).is_some());
    Ok(())
}
