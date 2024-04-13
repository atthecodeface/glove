//a Import modules

// Rename to kernel
mod accel_wgpu;
mod cpu;
mod kernel_args;
mod kernels;
pub use kernel_args::KernelArgs;
pub use kernels::Kernels;

mod traits;
pub use traits::Accelerate;
