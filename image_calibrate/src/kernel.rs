//a Import modules

// Rename to kernel
mod accel_wgpu;
mod cpu;
mod kernels;
pub use kernels::{KernelArgs, Kernels};

mod traits;
pub use traits::Accelerate;
