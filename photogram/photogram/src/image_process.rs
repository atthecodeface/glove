use anyhow::anyhow;
use thunderclap::{CmdDescriptor, CommandArgs};

use ic_photogram::{Image, ImageCache, ImageConvert, ImageDraw, ImageLuma16, ImageLumaF32};
use ic_photogram::{KernelArgs, Kernels};

use crate::cmd::{CmdArgs, CmdResult};

//a Help
//hi AS_LUMA_LONG_HELP
const AS_LUMA_LONG_HELP: &str = "\
Generate a 16-bit luma image
";

//hi LUMA_WINDOW_LONG_HELP
const LUMA_WINDOW_LONG_HELP: &str = "\
Analyze an image in luma space using a window
";

//hi LUMA_KERNEL_LONG_HELP
const LUMA_KERNEL_LONG_HELP: &str = "\
Analyze kernels to an image in luma space

Convert the image to a 16-bit luma

Apply a number of kernels (with a single set of size, scale etc arguments)

Output the image as a 16-bit luma image (so the kernel output should be in the range 0.0 to 1.0)
";

//hi LUMA_KERNEL_PAIR_LONG_HELP
const LUMA_KERNEL_PAIR_LONG_HELP: &str = "\
Apply kernels to a pair of images in luma space

Convert the images to 16-bit luma

Apply a number of kernels (with a single set of size, scale etc arguments)

Output the image as a 16-bit luma image (so the kernel output should be in the range 0.0 to 1.0)
";

impl CmdArgs {
    fn ip_as_luma_cmd(&mut self) -> CmdResult {
        let img_cref = self.get_read_image(0)?;

        let Some(img) = ImageCache::as_opt_luma16(&img_cref, None, 1.0) else {
            return Err(anyhow!("Could not generate luma16"));
        };
        if let Some(name) = self.cache_img.clone() {
            self.image_cache.add_image(name, img, true).unwrap();
        } else if let Some(write_filename) = self.write_img() {
            img.write(write_filename)?;
            eprintln!("Image {write_filename} written");
        }
        Self::cmd_ok()
    }

    fn ip_luma_kernel_cmd(&mut self) -> CmdResult {
        let img_cref = self.get_read_image(0)?;

        let Some(mut img_luma_f32) = ImageCache::as_opt_luma_f32(&img_cref, None, 1.0) else {
            return Err(anyhow!("Could not generate luma16"));
        };

        let ws = self.kernel_size();
        let scale = self.scale();
        let xy = self.pxy();
        let kernels_to_apply = self.kernels();

        let kernels = Kernels::new();
        let args: KernelArgs = img_luma_f32.dimensions().into();
        let args = args.with_size(ws);
        let args = args.with_scale(scale as f32);
        let args = args.with_xy(xy);
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

        if let Some(name) = self.cache_img.clone() {
            self.image_cache
                .add_image(name, img_luma_f32, true)
                .unwrap();
        } else if let Some(write_filename) = self.write_img() {
            let img = img_luma_f32.as_luma16(None, 1.0);
            img.write(write_filename)?;
            eprintln!("Image {write_filename} written");
        }
        Self::cmd_ok()
    }

    fn ip_luma_kernel_pair_cmd(&mut self) -> CmdResult {
        let img1_cref = self.get_read_image(0)?;
        let img2_cref = self.get_read_image(1)?;

        let Some(img1_luma_f32) = ImageCache::as_opt_luma_f32(&img1_cref, None, 1.0) else {
            return Err(anyhow!("Could not generate luma16"));
        };
        let Some(mut img2_luma_f32) = ImageCache::as_opt_luma_f32(&img2_cref, None, 1.0) else {
            return Err(anyhow!("Could not generate luma16"));
        };

        let ws = self.kernel_size();
        let scale = self.scale();
        let angle = self.angle();
        let xy = self.pxy();
        let flags = self.flags();
        let kernels_to_apply = self.kernels();

        let kernels = Kernels::new();

        let args: KernelArgs = img2_luma_f32.dimensions().into();
        let args = args.with_size(ws);
        let args = args.with_scale(scale as f32);
        let args = args.with_angle(angle.to_radians() as f32);
        let args = args.with_xy(xy);
        let args = args.with_src(img1_luma_f32.dimensions());

        for k in kernels_to_apply {
            kernels.run_shader(
                k,
                &args,
                img2_luma_f32.as_flat_samples().as_slice().len(),
                Some(img1_luma_f32.as_flat_samples().as_slice()),
                img2_luma_f32.as_flat_samples_mut().as_mut_slice(),
            )?;
        }

        if flags & 2 != 0 {
            let pts = kernels.find_best_n_above_value(
                img2_luma_f32.dimensions(),
                img2_luma_f32.as_flat_samples_mut().as_mut_slice(),
                500,
                0.7,
                64,
            );
            eprintln!("Points {pts:?}");
        }
        if let Some(name) = self.cache_img.clone() {
            self.image_cache
                .add_image(name, img2_luma_f32, true)
                .unwrap();
        } else if let Some(write_filename) = self.write_img() {
            let img = img2_luma_f32.as_luma16(None, 1.0);
            img.write(write_filename)?;
            eprintln!("Image {write_filename} written");
        }
        Self::cmd_ok()
    }

    const IP_AS_LUMA_CMD: CmdDescriptor<Self> = CmdDescriptor::new("as_luma")
        .about("Generate a 16-bit luma image")
        .long_about(AS_LUMA_LONG_HELP)
        .args(&[])
        .handler(&Self::ip_as_luma_cmd);

    const IP_LUMA_KERNEL_CMD: CmdDescriptor<Self> = CmdDescriptor::new("luma_kernel")
        .about("Apply kernels to an image in luma space")
        .long_about(LUMA_KERNEL_LONG_HELP)
        .args(&[
            Self::ARG_ADD_KERNEL,
            Self::ARG_SCALE,
            Self::ARG_KERNEL_SIZE,
            Self::ARG_PX,
            Self::ARG_PY,
        ])
        .handler(&Self::ip_luma_kernel_cmd);

    const IP_LUMA_KERNEL_PAIR_CMD: CmdDescriptor<Self> = CmdDescriptor::new("luma_kernel_pair")
        .about("Apply kernels to a pair of images in luma space")
        .long_about(LUMA_KERNEL_PAIR_LONG_HELP)
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
            Self::IP_LUMA_KERNEL_CMD,
            Self::IP_LUMA_KERNEL_PAIR_CMD,
        ]);
}
