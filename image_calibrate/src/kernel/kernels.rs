//a Imports
use crate::kernel::{accel_wgpu, cpu};
use crate::Accelerate;

//tp KernelArgs
#[derive(Debug, Default, Clone)]
pub struct KernelArgs {
    pub width: usize,
    pub height: usize,
    pub window_size: usize,
    pub scale: f32,
}

//ip From<(usize, usize)> for KernelArgs {
impl From<(usize, usize)> for KernelArgs {
    fn from((width, height): (usize, usize)) -> Self {
        Self {
            width,
            height,
            scale: 1.0,
            ..std::default::Default::default()
        }
    }
}

//ip KernelArgs
impl KernelArgs {
    pub fn with_window(mut self, window_size: usize) -> Self {
        self.window_size = window_size;
        self
    }
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
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
