//a Imports
use crate::{Accelerate, KernelArgs};

//tp ImageAccelerator
#[derive(Debug, Default)]
pub struct ImageAccelerator();

//ip ImageAccelerator
impl ImageAccelerator {
    //mp window_sum_y
    pub fn window_sum_y(&self, args: &KernelArgs, src_data: Option<&[u32]>, out_data: &mut [u32]) {
        let width = args.width;
        let height = args.height;
        let scale = args.scale;
        let mut col = vec![0_u32; height];
        let half_ws = args.window_size / 2;
        let skip = 2 * half_ws - 1;
        for x in 0..width {
            let src = &src_data.unwrap_or(out_data);
            let mut sum = 0;
            for y in 0..skip {
                sum += src[x + y * width];
            }
            for y in skip..height {
                sum += src[x + y * width];
                col[y - half_ws] = (sum * scale) >> 8;
                sum -= src[x + (y - skip) * width];
            }
            let v = col[half_ws];
            col[0..half_ws].fill(v);
            let v = col[height - 1 - half_ws];
            col[height - half_ws..height].fill(v);
            for y in 0..height {
                out_data[y * width + x] = col[y];
            }
        }
    }

    //mp window_sum_x
    pub fn window_sum_x(&self, args: &KernelArgs, src_data: Option<&[u32]>, out_data: &mut [u32]) {
        let width = args.width;
        let height = args.height;
        let scale = args.scale;
        let mut row = vec![0_u32; width];
        let half_ws = args.window_size / 2;
        let skip = 2 * half_ws - 1;
        for y in 0..height {
            let src_row = &src_data.unwrap_or(out_data)[y * width..(y * width + width)];
            let mut sum = 0;
            for x in 0..skip {
                sum += src_row[x];
            }
            for x in skip..width {
                sum += src_row[x];
                row[x - half_ws] = (sum * scale) >> 8;
                sum -= src_row[x - skip];
            }
            let v = row[half_ws];
            row[0..half_ws].fill(v);
            let v = row[width - 1 - half_ws];
            row[width - half_ws..width].fill(v);
            for x in 0..width {
                out_data[y * width + x] = row[x];
            }
        }
    }

    //mp add_scaled
    pub fn add_scaled(&self, args: &KernelArgs, src_data: Option<&[u32]>, out_data: &mut [u32]) {
        let width = args.width;
        let height = args.height;
        let scale = args.scale;
        if let Some(src_data) = src_data {
            for i in 0..width * height {
                out_data[i] = ((out_data[i] + src_data[i]) * scale) / 256;
            }
        }
    }

    //mp sub_scaled
    pub fn sub_scaled(&self, args: &KernelArgs, src_data: Option<&[u32]>, out_data: &mut [u32]) {
        let width = args.width;
        let height = args.height;
        let scale = args.scale;
        if let Some(src_data) = src_data {
            for i in 0..width * height {
                out_data[i] = (out_data[i].wrapping_sub(src_data[i]) * scale) / 256;
            }
        }
    }

    //mp square
    pub fn square(&self, args: &KernelArgs, src_data: Option<&[u32]>, out_data: &mut [u32]) {
        let width = args.width;
        let height = args.height;
        let scale = args.scale;
        if let Some(src_data) = src_data {
            for i in 0..width * height {
                out_data[i] = src_data[i] * src_data[i] / scale;
            }
        } else {
            for i in 0..width * height {
                out_data[i] = out_data[i] * out_data[i] / scale;
            }
        }
    }

    //mp sqrt
    pub fn sqrt(&self, args: &KernelArgs, src_data: Option<&[u32]>, out_data: &mut [u32]) {
        let width = args.width;
        let height = args.height;
        let scale = args.scale;
        if let Some(src_data) = src_data {
            for i in 0..width * height {
                out_data[i] = ((src_data[i] as f32).sqrt() as u32 * scale) >> 8;
            }
        } else {
            for i in 0..width * height {
                out_data[i] = ((out_data[i] as f32).sqrt() as u32 * scale) >> 8;
            }
        }
    }

    //zz All done
}

//ip Accelerate for ImageAccelerator
impl Accelerate for ImageAccelerator {
    //mp run_shader
    fn run_shader(
        &self,
        shader: &str,
        args: &KernelArgs,
        src_data: Option<&[u32]>,
        out_data: &mut [u32],
    ) -> Result<bool, String> {
        match shader {
            "window_sum_x" => {
                self.window_sum_x(args, src_data, out_data);
                Ok(true)
            }
            "window_sum_y" => {
                self.window_sum_y(args, src_data, out_data);
                Ok(true)
            }
            "add_scaled" => {
                self.add_scaled(args, src_data, out_data);
                Ok(true)
            }
            "sub_scaled" => {
                self.sub_scaled(args, src_data, out_data);
                Ok(true)
            }
            "square" => {
                self.square(args, src_data, out_data);
                Ok(true)
            }
            "sqrt" => {
                self.sqrt(args, src_data, out_data);
                Ok(true)
            }
            _ => Err(format!("Unimplemented shader {shader}")),
        }
    }
}
