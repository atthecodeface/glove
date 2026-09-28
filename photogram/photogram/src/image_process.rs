use thunderclap::{CmdDescriptor, CommandArgs};

use ic_photogram::{Image, ImageCache, ImageConvert, ImageDraw, ImageGray16, LumaF32Image};
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
    fn get_image_as_luma_f32(&mut self, n: usize) -> ic_photogram::Result<LumaF32Image> {
        let img = self.get_image_read_or_create(n)?;
        eprintln!(
            "Read initial image, size is {:?} (max pixels in kernel is 4M)",
            img.dimensions()
        );
        let (w, h) = img.dimensions();
        let npix = w as usize * h as usize;
        let max = 4 * 1024 * 1024;
        let scale =
            (npix > max).then_some(((max as f32 / npix as f32).sqrt() * w as f32).floor() as u32);
        let img_luma_f32 = img.as_luma_f32(scale, 1.0);
        eprintln!(
            "Using size {w}, {h} ({:.2} Mpx)",
            (w * h) as f32 / 1024.0 / 1024.0
        );
        Ok(img_luma_f32)
    }

    fn ip_as_luma_cmd(&mut self) -> CmdResult {
        let img = self.get_image_read_or_create(0)?;

        eprintln!("Read initial image, size is {:?}", img.dimensions());
        let img = img.as_luma16(None, 1.0);

        eprintln!("Created luma image");

        if let Some(write_filename) = self.write_img() {
            img.write(write_filename)?;
            eprintln!("Image written");
        } else {
            eprintln!("Image not written as no output image provided");
        }
        Self::cmd_ok()
    }

    fn ip_luma_window_cmd(&mut self) -> CmdResult {
        let mut img_luma_f32 = self.get_image_as_luma_f32(0)?;

        let kernels = Kernels::new();
        let ws = 8;
        let args: KernelArgs = img_luma_f32.dimensions().into();
        let args = args.with_size(ws as usize);
        let ws_f = ws as f32;
        let args_mean = args.with_scale(1.0 / ws_f);
        let num_pixels = img_luma_f32.as_flat_samples().as_slice().len();

        kernels.run_shader(
            "window_var",
            &args_mean,
            num_pixels,
            None,
            img_luma_f32.as_flat_samples_mut().as_mut_slice(),
        )?;

        eprintln!("Completed kernel");
        let img = img_luma_f32.as_luma16(None, 1.0);
        eprintln!("Created luma image");

        if let Some(write_filename) = self.write_img() {
            img.write(write_filename)?;
            eprintln!("Image written");
        } else {
            eprintln!("Image not written as no output image provided");
        }
        Ok("".into())
    }

    fn ip_luma_kernel_cmd(&mut self) -> CmdResult {
        let mut img_luma_f32 = self.get_image_as_luma_f32(0)?;

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

        eprintln!("Completed kernel");
        let img = img_luma_f32.as_luma16(None, 1.0);
        eprintln!("Created luma image");

        if let Some(write_filename) = self.write_img() {
            img.write(write_filename)?;
            eprintln!("Image written");
        } else {
            eprintln!("Image not written as no output image provided");
        }
        CmdArgs::cmd_ok()
    }

    fn ip_luma_kernel_pair_cmd(&mut self) -> CmdResult {
        let mut img1_luma_f32 = self.get_image_as_luma_f32(0)?;
        let mut img2_luma_f32 = self.get_image_as_luma_f32(1)?;

        let ws = self.kernel_size();
        let scale = self.scale();
        let angle = self.angle();
        let xy = self.pxy();
        let flags = self.flags();
        let kernels_to_apply = self.kernels();

        {
            let img = img1_luma_f32.as_luma16(None, 1.0);
            img.write("src_kernel.png")?;
        }

        {
            let img = img2_luma_f32.as_luma16(None, 1.0);
            img.write("dst_kernel.png")?;
        }

        let kernels = Kernels::new();

        if flags & 1 != 0 {
            eprintln!("Applying window_var_scaled to first");
            let args: KernelArgs = img1_luma_f32.dimensions().into();
            let args = args.with_size(4);
            kernels.run_shader(
                "window_var_scaled",
                &args,
                img1_luma_f32.as_flat_samples().as_slice().len(),
                None,
                img1_luma_f32.as_flat_samples_mut().as_mut_slice(),
            )?;
            eprintln!("Applying window_var_scaled to second");
            let args: KernelArgs = img2_luma_f32.dimensions().into();
            let args = args.with_size(4);
            kernels.run_shader(
                "window_var_scaled",
                &args,
                img2_luma_f32.as_flat_samples().as_slice().len(),
                None,
                img2_luma_f32.as_flat_samples_mut().as_mut_slice(),
            )?;
        }

        {
            let img = img2_luma_f32.as_luma16(None, 1.0);
            img.write("dst2_kernel.png")?;
        }

        let args: KernelArgs = img2_luma_f32.dimensions().into();
        let args = args.with_size(ws);
        let args = args.with_scale(scale as f32);
        let args = args.with_angle(angle.to_radians() as f32);
        let args = args.with_xy(xy);
        let args = args.with_src(img1_luma_f32.dimensions());

        for k in kernels_to_apply {
            {
                let img = img2_luma_f32.as_luma16(None, 1.0);
                img.write("dst3_kernel.png")?;
            }
            eprintln!("Applying {k} with {args:?}");
            {
                let img = img1_luma_f32.as_luma16(None, 1.0);
                img.write("before_src_kernel.png")?;
            }
            {
                let img = img2_luma_f32.as_luma16(None, 1.0);
                img.write("before_dst_kernel.png")?;
            }
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

        eprintln!("Completed kernel");
        let img = img2_luma_f32.as_luma16(None, 1.0);
        eprintln!("Created luma image");

        if let Some(write_filename) = self.write_img() {
            img.write(write_filename)?;
            eprintln!("Image written");
        } else {
            eprintln!("Image not written as no output image provided");
        }
        CmdArgs::cmd_ok()
    }

    const IP_AS_LUMA_CMD: CmdDescriptor<Self> = CmdDescriptor::new("as_luma")
        .about("Generate a 16-bit luma image")
        .long_about(AS_LUMA_LONG_HELP)
        .args(&[])
        .handler(&Self::ip_as_luma_cmd);

    const IP_LUMA_WINDOW_CMD: CmdDescriptor<Self> = CmdDescriptor::new("luma_window")
        .about("Analyze an image in luma space using a window")
        .long_about(LUMA_WINDOW_LONG_HELP)
        .args(&[])
        .handler(&Self::ip_luma_window_cmd);

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
            Self::ARG_READ_IMAGE_REQUIRED,
            Self::ARG_WRITE_IMAGE_OPTIONAL,
            Self::ARG_BG_COLOR,
        ])
        .cmds(&[
            Self::IP_AS_LUMA_CMD,
            Self::IP_LUMA_WINDOW_CMD,
            Self::IP_LUMA_KERNEL_CMD,
            Self::IP_LUMA_KERNEL_PAIR_CMD,
        ]);
}
