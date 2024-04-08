//a Imports
use std::io::Cursor;
use std::path::Path;

use crate::image::{Image, ImageRgb8};
use image::io::Reader as ImageReader;
pub use image::DynamicImage;

use image::{GenericImageView, ImageBuffer, Luma};

//a ImageGray16
#[derive(Debug, Clone)]
pub struct ImageGray16(DynamicImage);

//a ip ImageGray16
impl ImageGray16 {
    //cp read_image
    pub fn read_image<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let img: ImageBuffer<Luma<u16>, Vec<u16>> = ImageReader::open(path)
            .map_err(|e| format!("Failed to open file {}", e))?
            .decode()
            .map_err(|e| format!("Failed to decode image {}", e))?
            .into_luma16();
        Ok(Self(img.into()))
    }

    //fp read_or_create_image
    pub fn read_or_create_image(
        width: usize,
        height: usize,
        opt_filename: Option<&str>,
    ) -> Result<Self, String> {
        let width = width as u32;
        let height = height as u32;
        if let Some(filename) = opt_filename {
            let img = Self::read_image(filename)?;
            let (w, h) = img.size();
            if w != width || h != height {
                Err(format!(
                    "Image read has incorrect dimensions of ({},{}) instead of ({width},{height})",
                    w, h,
                ))
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

    //ac as_mut_slice
    pub(crate) fn as_mut_slice(&mut self) -> image::FlatSamples<&mut [u16]> {
        self.0.as_mut_luma16().unwrap().as_flat_samples_mut()
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

    //mp as_vec_u32
    pub fn as_vec_u32(&self, as_width: Option<usize>) -> (usize, usize, Vec<u32>) {
        let size = self.size();
        let size = (size.0 as usize, size.1 as usize);
        let (width, height) = as_width.map(|w| (w, w * size.1 / size.0)).unwrap_or(size);
        let mut result: Vec<u32> = vec![0; width * height];
        let s = self.as_slice();
        let mut i = 0;
        for y in 0..height {
            let sy = y * size.1 / height;
            let sy_ofs = sy * size.0;
            for x in 0..width {
                let sx = x * size.0 / width;
                result[i] = s[sy_ofs + sx] as u32;
                i += 1;
            }
        }
        (width, height, result)
    }

    //cp of_vec_u32
    pub fn of_vec_u32(width: usize, height: usize, data: Vec<u32>, scale: u32) -> Self {
        let data_u16: Vec<u16> = data.into_iter().map(|x| (x / scale) as u16).collect();
        let img =
            ImageBuffer::<Luma<u16>, Vec<u16>>::from_raw(width as u32, height as u32, data_u16)
                .unwrap();
        Self(img.into())
    }

    //mp window_sum_x
    pub fn window_sum_x(&mut self, window_size: usize, scale_down_shf_8: usize) {
        let (width, height) = self.size();
        let width = width as usize;
        let height = height as usize;
        let img = self.0.as_mut_luma16().unwrap();
        let mut ms = img.as_flat_samples_mut();
        let buf = ms.as_mut_slice();
        let mut row = vec![0_usize; width];
        let half_ws = window_size / 2;
        let skip = 2 * half_ws - 1;
        for y in 0..height {
            let buf_row = &mut buf[y * width..(y * width + width)];
            let mut sum = 0;
            for x in 0..width {
                let v: usize = buf_row[x] as usize;
                sum += v;
                if x >= skip {
                    row[x - half_ws] = sum;
                    let v: usize = buf_row[x - skip] as usize;
                    sum -= v;
                }
            }
            let v = row[half_ws];
            row[0..half_ws].fill(v);
            let v = row[width - 1 - half_ws];
            row[width - half_ws..width].fill(v);
            for x in 0..width {
                buf_row[x] = ((row[x] * scale_down_shf_8) >> 8) as u16;
            }
        }
    }

    //mp window_sum_y
    pub fn window_sum_y(&mut self, window_size: usize, scale_down_shf_8: usize) {
        let (width, height) = self.size();
        let width = width as usize;
        let height = height as usize;
        let img = self.0.as_mut_luma16().unwrap();
        let mut ms = img.as_flat_samples_mut();
        let buf = ms.as_mut_slice();
        let mut row = vec![0_usize; height];
        let half_ws = window_size / 2;
        let skip = 2 * half_ws - 1;
        for x in 0..width {
            let mut sum = 0;
            for y in 0..height {
                sum += buf[x + y * width] as usize;
                if y >= skip {
                    row[y - half_ws] = sum;
                    sum -= buf[x + (y - skip) * width] as usize;
                }
            }
            for y in half_ws..(height - half_ws) {
                buf[x + y * width] = ((row[y] * scale_down_shf_8) >> 8) as u16;
            }
            let v0 = ((row[half_ws] * scale_down_shf_8) >> 8) as u16;
            let v1 = ((row[height - 1 - half_ws] * scale_down_shf_8) >> 8) as u16;
            for y in 0..half_ws {
                buf[x + y * width] = v0;
                buf[x + (y + height - half_ws) * width] = v1;
            }
        }
    }

    //mp square
    pub fn square(&mut self, scale_down_shf_16: usize) {
        let (width, height) = self.size();
        let width = width as usize;
        let height = height as usize;
        let img = self.0.as_mut_luma16().unwrap();
        let mut ms = img.as_flat_samples_mut();
        let buf = ms.as_mut_slice();
        for i in 0..width * height {
            let v = buf[i] as usize;
            buf[i] = ((v * v * scale_down_shf_16) >> 16) as u16;
        }
    }

    //mp sqrt
    pub fn sqrt(&mut self, scale_down_shf_16: usize) {
        let (width, height) = self.size();
        let width = width as usize;
        let height = height as usize;
        let img = self.0.as_mut_luma16().unwrap();
        let mut ms = img.as_flat_samples_mut();
        let buf = ms.as_mut_slice();
        for i in 0..width * height {
            let v = buf[i] as f64;
            let v = v.sqrt() * (scale_down_shf_16 as f64) / 65536.0;
            buf[i] = v as u16;
        }
    }

    //mp add_scaled
    pub fn add_scaled(
        &mut self,
        other: &Self,
        scale_self: isize,
        scale_other: isize,
        scale_down_shf_16: usize,
    ) {
        let (width, height) = self.size();
        assert_eq!(other.size().0, width);
        assert_eq!(other.size().1, height);
        let width = width as usize;
        let height = height as usize;
        let img = self.0.as_mut_luma16().unwrap();
        let mut ms = img.as_flat_samples_mut();
        let buf = ms.as_mut_slice();
        let oimg = other.0.as_luma16().unwrap();
        let oms = oimg.as_flat_samples();
        let obuf = oms.as_slice();
        for i in 0..width * height {
            let v = buf[i] as isize;
            let wv = obuf[i] as isize;
            let v = (v * scale_self + wv * scale_other).max(0) as usize;
            buf[i] = ((v * scale_down_shf_16) >> 16) as u16;
        }
    }

    //zz All done
}

//ip Image for ImageGray16
impl Image for ImageGray16 {
    type Pixel = u16;
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        self.0
            .save(path)
            .map_err(|e| format!("Failed to encode image {}", e))?;
        Ok(())
    }
    fn encode(&self, extension: &str) -> Result<Vec<u8>, String> {
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
