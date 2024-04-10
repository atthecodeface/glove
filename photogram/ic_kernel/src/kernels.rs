//a Imports
use crate::kernel::{accel_wgpu, cpu};
use crate::Accelerate;

//tp KernelArgs
/// This type must be mappable for accelerators, hence u32 and f32
/// only
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct KernelArgs {
    /// Width of the 'image'
    pub width: u32,
    /// Height of the 'image'
    pub height: u32,
    /// Radius of a circle, window size, etc
    pub size: u32,
    /// Scale factor to apply (depends on kernel)
    pub scale: f32,
}

//ip From<(usize, usize)> for KernelArgs {
impl From<(usize, usize)> for KernelArgs {
    fn from((width, height): (usize, usize)) -> Self {
        Self {
            width: width as u32,
            height: height as u32,
            scale: 1.0,
            ..std::default::Default::default()
        }
    }
}

//ip KernelArgs
impl KernelArgs {
    pub fn with_size(mut self, size: usize) -> Self {
        self.size = size as u32;
        self
    }
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
    pub fn size(&self) -> usize {
        self.size as usize
    }
    pub fn width(&self) -> usize {
        self.width as usize
    }
    pub fn height(&self) -> usize {
        self.height as usize
    }
    pub fn scale(&self) -> f32 {
        self.scale
    }
    pub fn dims(&self) -> (usize, usize) {
        (self.width as usize, self.height as usize)
    }
}

//a Kernels
//tp Kernels
// Want this to be Clone, Sync, Send
#[derive(Debug)]
pub struct Kernels {
    wgpu: Option<accel_wgpu::ImageAccelerator>,
    cpu: cpu::ImageAccelerator,
}

//ip Kernels
impl Kernels {
    pub fn new() -> Self {
        let accelerator = accel_wgpu::AccelWgpu::new();
        let wgpu = {
            match accel_wgpu::ImageAccelerator::new(accelerator, 1024 * 1024) {
                Err(e) => {
                    eprintln!("Wgpu acceleration failed, not using that : {e}");
                    None
                }
                Ok(s) => Some(s),
            }
        };
        let cpu = cpu::ImageAccelerator::default();
        Self { wgpu, cpu }
    }

    //mp run_shader
    pub fn run_shader(
        &self,
        shader: &str,
        args: &KernelArgs,
        src_data: Option<&[f32]>,
        out_data: &mut [f32],
    ) -> Result<(), String> {
        if let Some(wgpu) = &self.wgpu {
            if wgpu.run_shader(shader, args, src_data, out_data)? {
                return Ok(());
            }
        }
        self.cpu
            .run_shader(shader, args, src_data, out_data)
            .map(|_| ())
    }
}
