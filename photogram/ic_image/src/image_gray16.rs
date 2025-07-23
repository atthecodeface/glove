//a Imports
use std::io::Cursor;
use std::path::Path;

use image::DynamicImage;
use image::ImageReader;
use image::{GenericImageView, ImageBuffer, Luma};

use ic_base::Result;

use crate::{Image, ImageRgb8};

//a ImageGray16
#[derive(Debug, Clone)]
pub struct ImageGray16(DynamicImage);

//a ip ImageGray16
impl ImageGray16 {
    //cp read_image
    pub fn read_image<P: AsRef<Path>>(path: P) -> Result<Self> {
        let img: ImageBuffer<Luma<u16>, Vec<u16>> =
            ImageReader::open(path)?.decode()?.into_luma16();
        Ok(Self(img.into()))
    }

    //fp read_or_create_image
    pub fn read_or_create_image(
        width: usize,
        height: usize,
        opt_filename: Option<&str>,
    ) -> Result<Self> {
        let width = width as u32;
        let height = height as u32;
        if let Some(filename) = opt_filename {
            let img = Self::read_image(filename)?;
            let (w, h) = img.size();
            if w != width || h != height {
                Err(format!(
                    "Image read has incorrect dimensions of ({},{}) instead of ({width},{height})",
                    w, h,
                )
                .into())
            } else {
                Ok(img)
            }
        } else {
            Ok(Self(DynamicImage::new_luma16(width, height)))
        }
    }

    //ac buffer
    pub(crate) fn buffer(&self) -> &image::DynamicImage {
        &self.0
    }

    //ac as_slice
    pub(crate) fn as_slice(&self) -> &[u16] {
        self.0.as_luma16().unwrap().as_raw()
    }

    //cp of_rgb
    pub fn of_rgb(image: &ImageRgb8, max: u32) -> Self {
        let (width, height) = image.size();
        let mut image_gray = ImageBuffer::<Luma<u16>, Vec<u16>>::new(width, height);
        let r_sc = 52 * max;
        let g_sc = 177 * max;
        let b_sc = 18 * max;
        for (x, y, rgba) in image.buffer().pixels() {
            let l = (rgba[0] as u32) * r_sc + (rgba[1] as u32) * g_sc + (rgba[2] as u32) * b_sc;
            let l = (l >> 16) as u16;
            image_gray[(x, y)] = [l].into();
        }
        Self(image_gray.into())
    }

    //mp as_vec_f32
    pub fn as_vec_f32(&self, as_width: Option<usize>) -> (usize, usize, Vec<f32>) {
        let size = self.size();
        let size = (size.0 as usize, size.1 as usize);
        let (width, height) = as_width.map(|w| (w, w * size.1 / size.0)).unwrap_or(size);
        let mut result: Vec<f32> = vec![0.0; width * height];
        let s = self.as_slice();
        let mut i = 0;
        for y in 0..height {
            let sy = y * size.1 / height;
            let sy_ofs = sy * size.0;
            for x in 0..width {
                let sx = x * size.0 / width;
                result[i] = s[sy_ofs + sx] as f32 / 65536.0;
                i += 1;
            }
        }
        (width, height, result)
    }

    //cp of_vec_f32
    pub fn of_vec_f32(width: usize, height: usize, data: Vec<f32>, scale: f32) -> Self {
        let data_u16: Vec<u16> = data
            .into_iter()
            .map(|x| ((x / scale) * 65535.9).floor() as u16)
            .collect();
        let img =
            ImageBuffer::<Luma<u16>, Vec<u16>>::from_raw(width as u32, height as u32, data_u16)
                .unwrap();
        Self(img.into())
    }

    //zz All done
}

//ip Image for ImageGray16
impl Image for ImageGray16 {
    type Pixel = u16;
    fn new(width: usize, height: usize) -> Self {
        Self(DynamicImage::new_luma16(width as u32, height as u32))
    }
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.0
            .save(path)
            .map_err(|e| format!("Failed to encode image {}", e))?;
        Ok(())
    }
    fn encode(&self, extension: &str) -> Result<Vec<u8>> {
        let format = {
            match extension {
                "jpg" => image::ImageFormat::Jpeg,
                "jpeg" => image::ImageFormat::Jpeg,
                "png" => image::ImageFormat::Png,
                _ => Err(format!("Unknown image format {extension}"))?,
            }
        };

        let mut bytes: Vec<u8> = Vec::new();
        self.0
            .write_to(&mut Cursor::new(&mut bytes), format)
            .map_err(|e| format!("Failed to encode image {}", e))?;
        Ok(bytes)
    }
    fn put(&mut self, x: u32, y: u32, color: &Self::Pixel) {
        let img = self.0.as_mut_luma16().unwrap();
        image::GenericImage::put_pixel(img, x, y, [*color].into());
    }
    fn get(&self, x: u32, y: u32) -> Self::Pixel {
        let img = self.0.as_luma16().unwrap();
        img.get_pixel(x, y).0[0]
    }
    fn size(&self) -> (u32, u32) {
        (self.0.width(), self.0.height())
    }
}
