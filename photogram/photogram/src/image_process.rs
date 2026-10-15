use anyhow::anyhow;
use thunderclap::{CmdDescriptor, CommandArgs, json};

use ic_photogram::{
    Image, ImageCache, ImageCacheEntry, ImageConvert, ImageDraw, ImageLuma16, ImageLumaF32, Result,
};
use ic_photogram::{KernelArgs, Kernels};

use crate::cmd::{CmdArgs, CmdResult};

impl CmdArgs {
    /// Get the KernelArgs with a given source image size
    fn ip_get_kernel_args<I: ImageDraw>(&self, img: &I, src_img: Option<&I>) -> KernelArgs {
        let mut args: KernelArgs = img.dimensions().into();

        let ws = self.kernel_size();
        args = args.with_size(ws);

        let scale = self.scale();
        args = args.with_scale(scale as f32);

        let xy = self.pxy();
        args = args.with_xy(xy);

        let angle = self.angle();
        args = args.with_angle(angle.to_radians() as f32);

        if let Some(src_img) = src_img {
            args = args.with_src(src_img.dimensions());
        }
        args
    }

    fn ip_handle_result_image<I: ImageConvert + Into<ImageCacheEntry> + std::fmt::Debug>(
        &mut self,
        img: I,
    ) -> Result<()> {
        if let Some(name) = self.cache_img.clone() {
            let cr = self.image_cache.add_image(name, img, true).unwrap();
            if let Some(write_filename) = self.write_img() {
                if let Some(img) = ImageCache::as_opt_luma16(&cr, None, 1.0) {
                    img.write(write_filename)?;
                }
            }
        } else if let Some(write_filename) = self.write_img() {
            let img = img.as_luma16(None, 1.0);
            img.write(write_filename)?;
        }
        Ok(())
    }

    const AS_LUMA_LONG_HELP: &str = "\
    Generate a 16-bit luma image from another image

    This is normally used to generate a luma cache internal image from an external JPEG

    photogram image_process --image colorful.jpg --cache luma_16 as_luma

    ";
    fn ip_as_luma_cmd(&mut self) -> CmdResult {
        let img_cref = self.get_read_image(0)?;

        let Some(img) = ImageCache::as_opt_luma16(&img_cref, None, 1.0) else {
            return Err(anyhow!("Could not generate luma16"));
        };
        self.ip_handle_result_image(img)?;
        Self::cmd_ok()
    }

    const LUMA_KERNEL_LONG_HELP: &str = "\
    Apply one ore more kernels to an (f32) image

    The image is first converted to an f32 luma image (if it was not already one)

    The supplied kernels are then applied in order

    Kernel arguments include:
     --px/py to provide x and y
     --angle <degrees> to provide angle in radians to the kernel
     --scale
     --size

    The result can be saved to the cache and/or written to an output image

    image_process --image luma_16 --cache processed luma_kernel --kernel equalize --kernel_size 32 --px 0 --py 0

    ";
    fn ip_luma_kernel_cmd(&mut self) -> CmdResult {
        let img_cref = self.get_read_image(0)?;

        let Some(mut img_luma_f32) = ImageCache::as_opt_luma_f32(&img_cref, None, 1.0) else {
            return Err(anyhow!("Could not generate luma16"));
        };

        let kernels_to_apply = self.kernels();

        let mut kernels = Kernels::default();
        let _ = kernels.add_wgpu_shaders("shaders", &["statistical", "extract"]);
        let args = self.ip_get_kernel_args(&img_luma_f32, None);
        let num_pixels = img_luma_f32.as_flat_samples().as_slice().len();

        for k in kernels_to_apply {
            kernels.run_shader(
                k,
                &args,
                num_pixels,
                None,
                img_luma_f32.as_flat_samples_mut().as_mut_slice(),
            )?;
        }
        self.ip_handle_result_image(img_luma_f32)?;
        Self::cmd_ok()
    }

    const LUMA_KERNEL_PAIR_LONG_HELP: &str = "\
    Apply kernels to a pair of images in luma space

    Convert the images to 16-bit luma

    Apply a number of kernels (with a single set of size, scale etc arguments)

    Output the image as a 16-bit luma image (so the kernel output should be in the range 0.0 to 1.0)
    ";
    fn ip_luma_kernel_pair_cmd(&mut self) -> CmdResult {
        let img1_cref = self.get_read_image(0)?;
        let img2_cref = self.get_read_image(1)?;

        let Some(img1_luma_f32) = ImageCache::as_opt_luma_f32(&img1_cref, None, 1.0) else {
            return Err(anyhow!("Could not generate luma16"));
        };
        let Some(mut img2_luma_f32) = ImageCache::as_opt_luma_f32(&img2_cref, None, 1.0) else {
            return Err(anyhow!("Could not generate luma16"));
        };

        let kernels_to_apply = self.kernels();

        let mut kernels = Kernels::default();
        let _ = kernels.add_wgpu_shaders("shaders", &["statistical", "extract"]);
        let args: KernelArgs = self.ip_get_kernel_args(&img2_luma_f32, Some(&img1_luma_f32));

        for k in kernels_to_apply {
            kernels.run_shader(
                k,
                &args,
                img2_luma_f32.as_flat_samples().as_slice().len(),
                Some(img1_luma_f32.as_flat_samples().as_slice()),
                img2_luma_f32.as_flat_samples_mut().as_mut_slice(),
            )?;
        }
        self.ip_handle_result_image(img2_luma_f32)?;
        Self::cmd_ok()
    }

    fn ip_luma_find_regions_cmd(&mut self) -> CmdResult {
        let img_cref = self.get_read_image(0)?;

        let Some(mut img_luma_f32) = ImageCache::as_opt_luma_f32(&img_cref, None, 1.0) else {
            return Err(anyhow!("Could not generate luma16"));
        };

        let mut kernels = Kernels::default();
        let _ = kernels.add_wgpu_shaders("shaders", &["statistical", "extract"]);

        let pts = kernels.find_best_n_above_value(
            img_luma_f32.dimensions(),
            img_luma_f32.as_flat_samples_mut().as_mut_slice(),
            500,                 // number
            self.scale() as f32, // value
            self.kernel_size(),  // min dist
        )?;
        Ok(json::to_value(pts)?)
    }

    const LUMA_KERNEL_RESIZE_LONG_HELP: &str = "\
    Apply a kernel to an input (f32) image with an output image of a given width/height

    The image is first converted to an f32 luma image (if it was not already one)

    The supplied kernels are then applied in order

    Kernel arguments include:
     --px/py to provide x and y
     --angle <degrees> to provide angle in radians to the kernel
     --scale
     --size

    The result can be saved to the cache and/or written to an output image

    image_process --image luma_16 --cache processed luma_kernel --kernel equalize --kernel_size 32 --px 0 --py 0

    ";
    fn ip_luma_kernel_resize_cmd(&mut self) -> CmdResult {
        let img1_cref = self.get_read_image(0)?;

        let Some(img1_luma_f32) = ImageCache::as_opt_luma_f32(&img1_cref, None, 1.0) else {
            return Err(anyhow!("Could not generate luma16"));
        };
        let mut img2_luma_f32 = ImageLumaF32::new(self.width, self.height);
        let kernels_to_apply = self.kernels();
        if kernels_to_apply.len() != 1 {
            return Err(anyhow!("Precisely one kernel should be provided"));
        }

        let mut kernels = Kernels::default();
        let _ = kernels.add_wgpu_shaders("shaders", &["statistical", "extract"]);
        let args = self.ip_get_kernel_args(&img2_luma_f32, Some(&img1_luma_f32));
        let num_pixels = img2_luma_f32.as_flat_samples().as_slice().len();

        for k in kernels_to_apply {
            kernels.run_shader(
                k,
                &args,
                num_pixels,
                Some(img1_luma_f32.as_flat_samples().as_slice()),
                img2_luma_f32.as_flat_samples_mut().as_mut_slice(),
            )?;
        }
        self.ip_handle_result_image(img2_luma_f32)?;
        Self::cmd_ok()
    }

    const IP_AS_LUMA_CMD: CmdDescriptor<Self> = CmdDescriptor::new("as_luma")
        .about("Generate a 16-bit luma image")
        .long_about(Self::AS_LUMA_LONG_HELP)
        .args(&[])
        .handler(&Self::ip_as_luma_cmd);

    const IP_LUMA_FIND_REGIONS_CMD: CmdDescriptor<Self> = CmdDescriptor::new("luma_find_regions")
        .about("Apply kernels to an image in luma space")
        .long_about(Self::LUMA_KERNEL_LONG_HELP)
        .args(&[Self::ARG_SCALE, Self::ARG_KERNEL_SIZE])
        .handler(&Self::ip_luma_find_regions_cmd);

    const IP_LUMA_KERNEL_RESIZE_CMD: CmdDescriptor<Self> = CmdDescriptor::new("luma_kernel_resize")
        .about("Apply a kernel to a source image, producing a destination image of a certain width/height")
        .long_about(Self::LUMA_KERNEL_RESIZE_LONG_HELP)
        .args(&[
            Self::ARG_ADD_KERNEL,
            Self::ARG_SCALE,
            Self::ARG_KERNEL_SIZE,
            Self::ARG_PX,
            Self::ARG_PY,
            Self::ARG_ANGLE,
            Self::ARG_FLAGS,
            Self::ARG_WIDTH,
            Self::ARG_HEIGHT,
        ])
        .handler(&Self::ip_luma_kernel_resize_cmd);

    const IP_LUMA_KERNEL_CMD: CmdDescriptor<Self> = CmdDescriptor::new("luma_kernel")
        .about("Apply kernels to an image in luma space")
        .long_about(Self::LUMA_KERNEL_LONG_HELP)
        .args(&[
            Self::ARG_ADD_KERNEL,
            Self::ARG_SCALE,
            Self::ARG_KERNEL_SIZE,
            Self::ARG_PX,
            Self::ARG_PY,
            Self::ARG_ANGLE,
            Self::ARG_FLAGS,
        ])
        .handler(&Self::ip_luma_kernel_cmd);

    const IP_LUMA_KERNEL_PAIR_CMD: CmdDescriptor<Self> = CmdDescriptor::new("luma_kernel_pair")
        .about("Apply kernels to a pair of images in luma space")
        .long_about(Self::LUMA_KERNEL_PAIR_LONG_HELP)
        .args(&[
            Self::ARG_ADD_KERNEL,
            Self::ARG_SCALE,
            Self::ARG_KERNEL_SIZE,
            Self::ARG_PX,
            Self::ARG_PY,
            Self::ARG_ANGLE,
            Self::ARG_FLAGS,
        ])
        .handler(&Self::ip_luma_kernel_pair_cmd);

    pub(crate) const IMAGE_PROCESS_CMD: CmdDescriptor<Self> = CmdDescriptor::new("image_process")
        .about("Perform image processing, such as applying kernels, to an image")
        .args(&[
            Self::ARG_IMAGE_OPTIONAL,
            Self::ARG_WRITE_IMAGE_OPTIONAL,
            Self::ARG_CACHE_IMAGE_OPTIONAL,
            Self::ARG_BG_COLOR,
        ])
        .cmds(&[
            Self::IP_AS_LUMA_CMD,
            Self::IP_LUMA_FIND_REGIONS_CMD,
            Self::IP_LUMA_KERNEL_CMD,
            Self::IP_LUMA_KERNEL_PAIR_CMD,
            Self::IP_LUMA_KERNEL_RESIZE_CMD,
        ]);
}
