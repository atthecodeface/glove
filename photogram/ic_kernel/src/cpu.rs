//a Imports
use crate::{Accelerate, KernelArgs};

//tp ImageAccelerator
#[derive(Debug, Default)]
pub struct ImageAccelerator();

//ip ImageAccelerator
impl ImageAccelerator {
    //mp window_sum_y
    pub fn window_sum_y(&self, args: &KernelArgs, src_data: Option<&[f32]>, out_data: &mut [f32]) {
        let width = args.width();
        let height = args.height();
        let scale = args.scale();
        let mut col = vec![0.0_f32; height];
        let half_ws = args.size() / 2;
        let skip = 2 * half_ws - 1;
        for x in 0..width {
            let src = &src_data.unwrap_or(out_data);
            let mut sum = 0.0;
            for y in 0..skip {
                sum += src[x + y * width];
            }
            for y in skip..height {
                sum += src[x + y * width];
                col[y - half_ws] = sum * scale;
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
    pub fn window_sum_x(&self, args: &KernelArgs, src_data: Option<&[f32]>, out_data: &mut [f32]) {
        let width = args.width();
        let height = args.height();
        let scale = args.scale();
        let mut row = vec![0.0_f32; width];
        let half_ws = args.size() / 2;
        let skip = 2 * half_ws - 1;
        for y in 0..height {
            let src_row = &src_data.unwrap_or(out_data)[y * width..(y * width + width)];
            let mut sum = 0.0;
            for s in src_row.iter().take(skip) {
                sum += s;
            }
            for x in skip..width {
                sum += src_row[x];
                row[x - half_ws] = sum * scale;
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
    pub fn add_scaled(&self, args: &KernelArgs, src_data: Option<&[f32]>, out_data: &mut [f32]) {
        let width = args.width();
        let height = args.height();
        let scale = args.scale();
        if let Some(src_data) = src_data {
            for i in 0..width * height {
                out_data[i] = (out_data[i] + src_data[i]) * scale;
            }
        }
    }

    //mp sub_scaled
    pub fn sub_scaled(&self, args: &KernelArgs, src_data: Option<&[f32]>, out_data: &mut [f32]) {
        let width = args.width();
        let height = args.height();
        let scale = args.scale();
        if let Some(src_data) = src_data {
            for i in 0..width * height {
                out_data[i] = (out_data[i] - src_data[i]) * scale;
            }
        }
    }

    //mp square
    pub fn square(&self, args: &KernelArgs, src_data: Option<&[f32]>, out_data: &mut [f32]) {
        let width = args.width();
        let height = args.height();
        let scale = args.scale();
        if let Some(src_data) = src_data {
            for i in 0..width * height {
                out_data[i] = src_data[i] * src_data[i] * scale;
            }
        } else {
            for od in out_data.iter_mut().take(width * height) {
                *od = *od * *od * scale;
            }
        }
    }

    //mp sqrt
    pub fn sqrt(&self, args: &KernelArgs, src_data: Option<&[f32]>, out_data: &mut [f32]) {
        let width = args.width();
        let height = args.height();
        let scale = args.scale();
        if let Some(src_data) = src_data {
            for i in 0..width * height {
                out_data[i] = src_data[i].sqrt() * scale;
            }
        } else {
            for od in out_data.iter_mut().take(width * height) {
                *od = od.sqrt() * scale;
            }
        }
    }

    //mp circle_fft16
    /*
    pub fn circle_fft16(&self, args: &KernelArgs, src_data: Option<&[f32]>, out_data: &mut [f32]) {
        let width = args.width();
        let height = args.height();
        let radius = args.size();
        let circle = [0.0_f32; 16];
        let mut dxy = vec![0_usize; 32];
        for angle in 0..16 {
            let angle = std::f32::consts::TAU * (angle as f32) / 16.0;
            let dx = (radius as f32) * angle.cos();
            let dy = (radius as f32) * angle.sin();
            dxy.push(dx.round() as usize);
            dxy.push(dy.round() as usize);
        }
                if let Some(src_data) = src_data {
                    for y in radius..(height-radius) {
                        for x in radius..(width-radius) {
                            for i in 0..16 {
                                circle[i] = src_data[x+dxy[2*i+0]+(y+dxy[2*i+1])*width];
                            }
                        }
                        fft[n] = sum(circle[i] * rot[n*i/16]);
                        fft[8] = sum(circle[i] * rot[8*i/16])
                        fft[0_rev] = sum(circle[i]);
                        fft[1_rev] = sum(circle[even i]) - sum(circle[odd_i]);
                        fft[2_rev] = sum(circle[4n]) - sum(circle[4n+2]), i*sum(circle[4n+1]) - sum(circle[4n+3]);
                        out_data[i] = 0.0;
                    }
                }
    }
        */

    //zz All done
}

//ip Accelerate for ImageAccelerator
impl Accelerate for ImageAccelerator {
    //mp run_shader
    fn run_shader(
        &self,
        shader: &str,
        args: &KernelArgs,
        _work_items: usize,
        src_data: Option<&[f32]>,
        out_data: &mut [f32],
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
