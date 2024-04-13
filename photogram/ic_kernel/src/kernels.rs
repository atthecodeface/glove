//a Imports
use crate::{accel_wgpu, cpu, Accelerate, KernelArgs};

//a Kernels
//tp Kernels
// Want this to be Clone, Sync, Send
#[derive(Debug)]
pub struct Kernels {
    wgpu: Option<accel_wgpu::ImageAccelerator>,
    cpu: cpu::ImageAccelerator,
}

//ip Default for Kernels
impl Default for Kernels {
    fn default() -> Self {
        Self::new()
    }
}
//ip Kernels
impl Kernels {
    pub fn new() -> Self {
        let accelerator = accel_wgpu::AccelWgpu::new();
        let wgpu = {
            match accel_wgpu::ImageAccelerator::new(accelerator, 16 * 1024 * 1024) {
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
