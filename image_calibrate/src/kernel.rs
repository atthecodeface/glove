//a Import modules

// Rename to kernel
mod cpu;

mod accel_wgpu;

// Want this to be Clone, Sync, Send
#[derive(Debug)]
pub struct Kernels {
    wgpu: Option<accel_wgpu::ImageAccelerator>,
    cpu: cpu::ImageAccelerator,
}

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
        args: &AccelerateArgs,
        src_data: Option<&[u32]>,
        out_data: &mut [u32],
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

mod traits;
pub use traits::{Accelerate, AccelerateArgs};
