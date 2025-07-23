//a Imports
use std::path::Path;

use crate::{accel_wgpu, cpu, Accelerate, KernelArgs};

//a Kernels
//tp Kernels
// Want this to be Clone, Sync, Send
#[derive(Debug)]
pub struct Kernels {
    wgpu: Option<accel_wgpu::ImageAccelerator>,
    cpu: cpu::ImageAccelerator,
    verbose: bool,
}

//ip Default for Kernels
impl Default for Kernels {
    fn default() -> Self {
        Self::new()
    }
}
//ip Kernels
impl Kernels {
    //ci read_json
    /// TODO: move me to accel_wgpu
    fn read_json<P: AsRef<Path>>(
        root: P,
        names: &[&str],
    ) -> Result<accel_wgpu::ImageAccelerator, String> {
        let accelerator = accel_wgpu::AccelWgpu::new();
        let mut wgpu = accel_wgpu::ImageAccelerator::new(accelerator, 16 * 1024 * 1024)?;
        for n in names {
            let mut root = root.as_ref().to_owned();
            root.push(n);
            let mut x = root.clone();
            x.set_extension("json");
            let json = std::fs::read_to_string(&x)
                .map_err(|e| format!("Error reading Json describing shader file {}", e))?;
            root.set_extension("wgsl");
            let x = root.clone();
            let shader_desc_file = accel_wgpu::ShaderFileDesc::from_json(x, &json)
                .map_err(|e| format!("Error parsing JSON file {}: {}", root.display(), e))?;
            wgpu.create_pipelines(shader_desc_file)?;
        }
        Ok(wgpu)
    }

    //cp new
    pub fn new() -> Self {
        let cpu = cpu::ImageAccelerator::default();
        let wgpu = {
            match Self::read_json("shaders", &["statistical", "extract"]) {
                Err(e) => {
                    eprintln!("Wgpu acceleration failed, not using that : {e}");
                    None
                }
                Ok(s) => Some(s),
            }
        };
        let verbose = false;
        Self { wgpu, cpu, verbose }
    }

    //mp set_verbose
    #[allow(dead_code)]
    fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    //mp run_shader
    pub fn run_shader(
        &self,
        shader: &str,
        args: &KernelArgs,
        work_items: usize,
        src_data: Option<&[f32]>,
        out_data: &mut [f32],
    ) -> Result<(), String> {
        if self.verbose {
            eprintln!("Run shader {shader} with {work_items} items");
        }
        if let Some(wgpu) = &self.wgpu {
            if wgpu.run_shader(shader, args, work_items, src_data, out_data)? {
                return Ok(());
            }
        }
        self.cpu
            .run_shader(shader, args, work_items, src_data, out_data)
            .map(|_| ())
    }

    //mp find_best_n_above_value
    pub fn find_best_n_above_value(
        &self,
        size: (usize, usize),
        data: &mut [f32],
        n: usize,
        value: f32,
        min_dist: usize,
    ) -> Result<Vec<(u32, u32, f32)>, String> {
        let mut result: Vec<(u32, u32, f32)> = vec![];
        loop {
            let new_pts = self.find_next_best_above_x(size, min_dist, data, value)?;
            if new_pts.is_empty() {
                result.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
                result.truncate(n);
                break;
            }
            self.mask_out_selected_points(
                size,
                data,
                new_pts.iter().map(|a| [a.0 as f32, a.1 as f32]),
                min_dist * 2,
            )?;
            let best_new_value = new_pts[0].2;
            if result.len() >= n {
                result.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
                result.truncate(n);
                if best_new_value < result[n - 1].2 {
                    break;
                }
            }
            result.extend(new_pts);
        }
        Ok(result)
    }

    //fi find_max_above_value_of_regions
    /// Returns a vec of region x y, data x y, value
    fn find_max_above_value_of_regions(
        &self,
        size: (usize, usize),
        data: &[f32],
        region_size: usize,
        min_value: f32,
    ) -> Result<Vec<(usize, usize, usize, usize, f32)>, String> {
        let (width, height) = size;
        let mut max_of_region_slice: Vec<f32> = data.into();
        let args: KernelArgs = size.into();
        let args = args.with_scale(min_value);
        let args = args.with_size(region_size);
        self.run_shader(
            "max_of_region",
            &args,
            width * height,
            None,
            &mut max_of_region_slice,
        )?;
        let mut centers_found: Vec<(usize, usize, usize, usize, f32)> = vec![];
        for y in (0..height).step_by(region_size) {
            for x in (0..width).step_by(region_size) {
                let idx = x + y * width;
                let ofs = bytemuck::cast::<f32, u32>(max_of_region_slice[idx]) as usize;
                let value = max_of_region_slice[idx + 1];
                let n = max_of_region_slice[idx + 2];
                if n > 0.0 {
                    centers_found.push((
                        x / region_size,
                        y / region_size,
                        ofs % width,
                        ofs / width,
                        value,
                    ));
                }
            }
        }
        Ok(centers_found)
    }

    //fi find_next_best_above_x
    /// Returns a Vec of <x,y,value> *sorted* by descending value
    fn find_next_best_above_x(
        &self,
        size: (usize, usize),
        min_dist: usize,
        data: &[f32],
        value: f32,
    ) -> Result<Vec<(u32, u32, f32)>, String> {
        // Find the max values splitting the data into regions no smaller than min_dist
        //
        // If two points are found in non-adjacent regions then they
        // MUST be more than min_dist apart
        //
        // The smaller the region size, the larger the regions found
        // will be, so this is a trade-off somewhat between wasted
        // work done in the kernel and fewer iterations
        //
        // If the expectation is one point will exceed the value then
        // the region size should be large; if many are expected then
        // a smaller region size makes sense
        let region_size = {
            if min_dist < 32 {
                32
            } else {
                min_dist
            }
        };
        let mut regions_found =
            self.find_max_above_value_of_regions(size, data, region_size, value)?;
        if regions_found.is_empty() {
            return Ok(vec![]);
        }

        // Order the regions by descending max value
        regions_found.sort_by(|a, b| (b.4).partial_cmp(&a.4).unwrap());

        // Select points fromm regions by casting out those that
        // neighbor regions with a *higher* max value
        //
        // If a regions is cast out, then still ignore all of its
        // neighboring regions with a lower max value, as this regions
        // may be permitted in a later iteration
        let mut selected_points: Vec<(u32, u32, f32)> = vec![];
        for (i, (rx, ry, x, y, value)) in regions_found.iter().enumerate() {
            let mut ignore = false;
            for (orx, ory, _, _, _) in &regions_found[0..i] {
                if (*rx == *orx || *rx == *orx + 1 || *rx + 1 == *orx)
                    && (*ry == *ory || *ry == *ory + 1 || *ry + 1 == *ory)
                {
                    ignore = true;
                    break;
                }
            }
            if !ignore {
                selected_points.push((*x as u32, *y as u32, *value));
            }
        }

        Ok(selected_points)
    }

    //fi mask_out_selected_points
    /// Updates an [f32] by masking down a set of selected points to a *diameter* of region_size
    ///
    /// First copy the current data to the 'out' buffer
    ///
    /// Then run a shader that just overwrites part of the 'out'
    /// buffer at the selected points with lower values
    fn mask_out_selected_points<I>(
        &self,
        size: (usize, usize),
        data: &mut [f32],
        selected_points: I,
        region_size: usize,
    ) -> Result<(), String>
    where
        I: Iterator<Item = [f32; 2]>,
    {
        let (width, height) = size;
        let args: KernelArgs = size.into();
        let args = args.with_scale(0.0);
        let args = args.with_cos(0.0);
        let args = args.with_size(region_size);
        self.run_shader("copy", &args, width * height, None, data)?;
        let things_to_reduce: Vec<f32> = selected_points.flatten().collect();
        self.run_shader(
            "reduce_value",
            &args,
            things_to_reduce.len() / 2,
            Some(&things_to_reduce),
            data,
        )?;
        Ok(())
    }

    //zz All done
}
