use ic_base::Result;
use ic_image::ImageLumaF32;
use ic_kernel::{KernelArgs, Kernels};

#[derive(Default)]
pub struct ImageProcessF32 {
    kernels: Kernels,
    kernel_args: KernelArgs,
    /// The main image to process
    image: ImageLumaF32,
    /// The reference image, if this has a 'parent'
    ref_image: Option<ImageLumaF32>,
    /// Images used during operations
    op_images: Vec<ImageLumaF32>,
}

impl ImageProcessF32 {
    fn run_image_pair_kernel(
        &self,
        kernel_name: &str,
        in_img: &ImageLumaF32,
        in_out_img: &mut ImageLumaF32,
    ) -> Result<()> {
        let num_pixels = in_out_img.as_flat_samples().as_slice().len();
        self.kernels.run_shader(
            kernel_name,
            &self.kernel_args,
            num_pixels,
            Some(in_img.as_flat_samples().as_slice()),
            in_out_img.as_flat_samples_mut().as_mut_slice(),
        )?;
        Ok(())
    }
}
