//a Imports
use std::rc::Rc;

use image::{GenericImage, GenericImageView};

use ic_base::Rrc;

use crate::{Image, ImageDrawable};

//a Consts
pub const SQUARE_SIZE: u32 = 8;

//a ImageSquareSet
//tp ImageSquareSet
#[derive(Debug)]
pub struct ImageSquareSet<I: Image> {
    image_filename: String,
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
    pub fn image_filename(&self) -> &str {
        &self.image_filename
    }

    //mp set_image_filename
    pub fn set_image_filename<S: Into<String>>(&mut self, s: S) {
        self.image_filename = s.into();
    }

    //cp create
    pub fn create(width_sq: u32, height_sq: u32) -> Self {
        let image = I::new(
            (width_sq * SQUARE_SIZE) as usize,
            (height_sq * SQUARE_SIZE) as usize,
        )
        .into();
        let used_squares_size = (width_sq * height_sq + 63) / 64;
        let used_squares = vec![0_u64; used_squares_size as usize];
        let used_squares = used_squares.into_boxed_slice();
        let image_filename = String::new();
        Self {
            image_filename,
            width_sq,
            height_sq,
            used_squares,
            image,
        }
    }

    //mp alloc_set_bit
    fn alloc_set_bit(&mut self, x: u32, y: u32) {
        let idx = (y * self.width_sq + x) as usize;
        let idx_bit = idx & 63;
        let idx = idx >> 6;
        self.used_squares[idx] |= 1 << (idx_bit as u64);
    }

    //mp alloc_clr_bit
    fn alloc_clr_bit(&mut self, x: u32, y: u32) {
        let idx = (y * self.width_sq + x) as usize;
        let idx_bit = idx & 63;
        let idx = idx >> 6;
        self.used_squares[idx] &= !(1 << (idx_bit as u64));
    }

    //mp alloc_get_bit
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

    //mi is_region_free
    fn is_region_free(&self, sq: u32, w_sq: u32, h_sq: u32) -> bool {
        let x_sq = sq % self.width_sq;
        let y_sq = sq / self.width_sq;
        if x_sq > self.width_sq - w_sq {
            false
        } else if y_sq > self.height_sq - h_sq {
            false
        } else {
            for y in 0..h_sq {
                for x in 0..w_sq {
                    if self.alloc_get_bit(x_sq + x, y_sq + y) {
                        return false;
                    }
                }
            }
            true
        }
    }

    //mi find_free_region
    fn find_free_region(&self, w_sq: u32, h_sq: u32) -> Option<(u32, u32)> {
        if w_sq * h_sq == 1 {
            if let Some(sq) = self.find_first_free_square_from(0) {
                Some((sq % self.width_sq, sq / self.width_sq))
            } else {
                None
            }
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

    //mi mark_alloc
    fn mark_alloc(&mut self, x_sq: u32, y_sq: u32, w_sq: u32, h_sq: u32) {
        for y in 0..h_sq {
            for x in 0..w_sq {
                self.alloc_set_bit(x_sq + x, y_sq + y);
            }
        }
    }

    //mi mark_free
    fn mark_free(&mut self, x_sq: u32, y_sq: u32, w_sq: u32, h_sq: u32) {
        for y in 0..h_sq {
            for x in 0..w_sq {
                self.alloc_clr_bit(x_sq + x, y_sq + y);
            }
        }
    }

    //mp allocate_squares
    #[track_caller]
    pub fn allocate_squares(&mut self, w: u32, h: u32) -> Option<ImageSquares<I>> {
        assert!(w.is_multiple_of(SQUARE_SIZE));
        assert!(h.is_multiple_of(SQUARE_SIZE));
        if let Some((x_sq, y_sq)) = self.find_free_region(w / SQUARE_SIZE, h / SQUARE_SIZE) {
            Some(self.select_squares(x_sq, y_sq, w, h, true))
        } else {
            None
        }
    }

    //mp free_squares
    #[track_caller]
    pub fn free_squares(&mut self, sqs: ImageSquares<I>) {
        let (x_sq, y_sq, w, h) = sqs.take();
        self.mark_free(x_sq, y_sq, w / SQUARE_SIZE, h / SQUARE_SIZE)
    }

    //mp select_squares
    #[track_caller]
    pub fn select_squares(
        &mut self,
        x_sq: u32,
        y_sq: u32,
        w: u32,
        h: u32,
        mark_alloc: bool,
    ) -> ImageSquares<I> {
        assert!(w.is_multiple_of(SQUARE_SIZE));
        assert!(h.is_multiple_of(SQUARE_SIZE));
        if mark_alloc {
            self.mark_alloc(x_sq, y_sq, w / SQUARE_SIZE, h / SQUARE_SIZE);
        }
        ImageSquares::selected_squares(self, x_sq, y_sq, w, h)
    }
}

//a ImageSquares
//tp ImageSquares
pub struct ImageSquares<I: Image> {
    image: Rrc<I>,
    w: u32,
    h: u32,
    x_sq: u32,
    y_sq: u32,
}

//ip Debug for ImageSquares
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
            self.x_sq * SQUARE_SIZE,
            self.y_sq * SQUARE_SIZE
        )
    }
}

//ip ImageSquares
impl<I> ImageSquares<I>
where
    I: Image,
{
    //dp take
    pub fn take(self) -> (u32, u32, u32, u32) {
        (self.x_sq, self.y_sq, self.w, self.h)
    }

    //cp selected_squares
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

    //mp copy_from_image
    #[track_caller]
    pub fn copy_from_image(&self, image: &I, x: u32, y: u32)
    where
        I: std::ops::Deref<Target = image::DynamicImage>,
        I: std::ops::DerefMut,
    {
        self.image
            .borrow_mut()
            .copy_from(
                &*image.view(x, y, self.w, self.h),
                self.x_sq * SQUARE_SIZE,
                self.y_sq * SQUARE_SIZE,
            )
            .unwrap();
    }

    //mp copy_to_image
    #[track_caller]
    pub fn copy_to_image(&self, image: &mut I, x: u32, y: u32)
    where
        I: std::ops::Deref<Target = image::DynamicImage>,
        I: std::ops::DerefMut,
    {
        image
            .copy_from(
                &*self.image.borrow().view(
                    self.x_sq * SQUARE_SIZE,
                    self.y_sq * SQUARE_SIZE,
                    self.w,
                    self.h,
                ),
                x,
                y,
            )
            .unwrap();
    }

    //mp copy_squares
    #[track_caller]
    pub fn copy_squares(&self, from: &Self)
    where
        I: std::ops::Deref<Target = image::DynamicImage>,
        I: std::ops::DerefMut,
    {
        assert_eq!(self.w, from.w);
        assert_eq!(self.h, from.h);
        if Rc::ptr_eq(&self.image, &from.image) {
            self.image.borrow_mut().copy_within(
                image::math::Rect {
                    x: from.x_sq * SQUARE_SIZE,
                    y: from.y_sq * SQUARE_SIZE,
                    width: self.w,
                    height: self.h,
                },
                self.x_sq * SQUARE_SIZE,
                self.y_sq * SQUARE_SIZE,
            );
        } else {
            self.copy_from_image(
                &from.image.borrow(),
                from.x_sq * SQUARE_SIZE,
                from.y_sq * SQUARE_SIZE,
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
        self.image
            .borrow()
            .get(x + (self.x_sq * SQUARE_SIZE), y + (self.y_sq * SQUARE_SIZE))
    }
    fn put(&mut self, x: u32, y: u32, color: &Self::Pixel) {
        self.image.borrow_mut().put(
            x + (self.x_sq * SQUARE_SIZE),
            y + (self.y_sq * SQUARE_SIZE),
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
    let mut set = ImageSquareSet::<crate::ImageRgb8>::create(10, 20);
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
    let mut set = ImageSquareSet::<crate::ImageRgb8>::create(10, 20);
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
    let mut set = ImageSquareSet::<crate::ImageRgb8>::create(10, 20);
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
