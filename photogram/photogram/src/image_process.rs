use thunderclap::{CmdDescriptor, CommandArgs};

use ic_image::{Image, ImageDrawable, ImageGray16};
use ic_kernel::{KernelArgs, Kernels};

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
        let img = self.get_image_read_or_create()?;

        eprintln!("Read initial image, size is {:?}", img.size());
        let (w, h, img_data) = img.as_vec_gray_f32(None);

        let img = ImageGray16::of_vec_f32(w, h, img_data, 1.0);

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
        let img = self.get_image_read_or_create()?;

        eprintln!(
            "Read initial image, size is {:?} (max pixels in kernel is 4M)",
            img.size()
        );
        let (w, h) = img.size();
        let npix = w as usize * h as usize;
        let max = 4 * 1024 * 1024;
        let scale =
            (npix > max).then_some(((max as f32 / npix as f32).sqrt() * w as f32).floor() as usize);
        let (w, h, mut img_data) = img.as_vec_gray_f32(scale);
        eprintln!(
            "Using size {w}, {h} ({:.2} Mpx)",
            (w * h) as f32 / 1024.0 / 1024.0
        );

        let kernels = Kernels::new();
        let ws = 8;
        let args: KernelArgs = (w, h).into();
        let args = args.with_size(ws as usize);
        let ws_f = ws as f32;
        let args_mean = args.with_scale(1.0 / ws_f);

        kernels.run_shader(
            "window_var",
            &args_mean,
            w * h,
            None,
            img_data.as_mut_slice(),
        )?;

        eprintln!("Completed kernel");
        let img = ImageGray16::of_vec_f32(w, h, img_data, 1.0);
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
        let img = self.get_read_image(0)?;

        let ws = self.kernel_size();
        let scale = self.scale();
        let xy = self.pxy();
        let kernels_to_apply = self.kernels();

        eprintln!(
            "Read initial image, size is {:?} (max pixels in kernel is 4M)",
            img.size()
        );
        let (w, h) = img.size();
        let npix = w as usize * h as usize;
        let max = 4 * 1024 * 1024;
        let img_scale =
            (npix > max).then_some(((max as f32 / npix as f32).sqrt() * w as f32).floor() as usize);
        let (w, h, mut img_data) = img.as_vec_gray_f32(img_scale);
        eprintln!(
            "Using size {w}, {h} ({:.2} Mpx)",
            (w * h) as f32 / 1024.0 / 1024.0
        );

        let kernels = Kernels::new();
        let args: KernelArgs = (w, h).into();
        let args = args.with_size(ws);
        let args = args.with_scale(scale as f32);
        let args = args.with_xy(xy);

        for k in kernels_to_apply {
            kernels.run_shader(k, &args, w * h, None, img_data.as_mut_slice())?;
        }

        eprintln!("Completed kernel");
        let img = ImageGray16::of_vec_f32(w, h, img_data, 1.0);
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
        let img2 = self.get_read_image(1)?;
        let img1 = self.get_read_image(0)?;

        let ws = self.kernel_size();
        let scale = self.scale();
        let angle = self.angle();
        let xy = self.pxy();
        let flags = self.flags();
        let kernels_to_apply = self.kernels();

        eprintln!(
            "Read initial image, size is {:?} (max pixels in kernel is 4M) : xy {xy:?}",
            img1.size()
        );

        let (src_w, src_h) = img1.size();
        let src_npix = src_w as usize * src_h as usize;
        let src_max = (4 * 1024 * 1024).min(src_npix);
        let src_img_scale =
            Some(((src_max as f32 / src_npix as f32).sqrt() * src_w as f32).floor() as usize);
        let (src_w, src_h, mut src_img) = img1.as_vec_gray_f32(src_img_scale);
        eprintln!(
            "Using size {src_w}, {src_h} ({:.2} Mpx)",
            (src_w * src_h) as f32 / 1024.0 / 1024.0
        );
        {
            let img = ImageGray16::of_vec_f32(src_w, src_h, src_img.clone(), 1.0);
            img.write("src_kernel.png")?;
        }

        let (dst_w, dst_h) = img1.size();
        let dst_npix = dst_w as usize * dst_h as usize;
        let dst_max = (4 * 1024 * 1024).min(dst_npix);
        let dst_img_scale =
            Some(((dst_max as f32 / dst_npix as f32).sqrt() * dst_w as f32).floor() as usize);
        let (dst_w, dst_h, mut img_data) = img2.as_vec_gray_f32(dst_img_scale);
        eprintln!(
            "Other size {dst_w}, {dst_h} ({:.2} Mpx)",
            (dst_w * dst_h) as f32 / 1024.0 / 1024.0
        );
        {
            let img = ImageGray16::of_vec_f32(dst_w, dst_h, img_data.clone(), 1.0);
            img.write("dst_kernel.png")?;
        }

        let kernels = Kernels::new();

        if flags & 1 != 0 {
            eprintln!("Applying window_var_scaled to first");
            let args: KernelArgs = (src_w, src_h).into();
            let args = args.with_size(4);
            kernels.run_shader(
                "window_var_scaled",
                &args,
                src_w * src_h,
                None,
                src_img.as_mut_slice(),
            )?;
            eprintln!("Applying window_var_scaled to second");
            let args: KernelArgs = (dst_w, dst_h).into();
            let args = args.with_size(4);
            kernels.run_shader(
                "window_var_scaled",
                &args,
                dst_w * dst_h,
                None,
                img_data.as_mut_slice(),
            )?;
        }

        {
            let img = ImageGray16::of_vec_f32(dst_w, dst_h, img_data.clone(), 1.0);
            img.write("dst2_kernel.png")?;
        }
        let args: KernelArgs = (dst_w, dst_h).into();
        let args = args.with_size(ws);
        let args = args.with_scale(scale as f32);
        let args = args.with_angle(angle.to_radians() as f32);
        let args = args.with_xy(xy);
        let args = args.with_src((src_w, src_h));

        for k in kernels_to_apply {
            {
                let img = ImageGray16::of_vec_f32(dst_w, dst_h, img_data.clone(), 1.0);
                img.write("dst3_kernel.png")?;
            }
            eprintln!("Applying {k} with {args:?}");
            {
                let img = ImageGray16::of_vec_f32(src_w, src_h, src_img.clone(), 1.0);
                img.write("before_src_kernel.png")?;
            }
            {
                let img = ImageGray16::of_vec_f32(dst_w, dst_h, img_data.clone(), 1.0);
                img.write("before_dst_kernel.png")?;
            }
            kernels.run_shader(
                k,
                &args,
                dst_w * dst_h,
                Some(src_img.as_slice()),
                img_data.as_mut_slice(),
            )?;
        }

        if flags & 2 != 0 {
            let pts = kernels.find_best_n_above_value(
                (dst_w, dst_h),
                img_data.as_mut_slice(),
                500,
                0.7,
                64,
            );
            eprintln!("Points {pts:?}");
        }

        eprintln!("Completed kernel");
        let img = ImageGray16::of_vec_f32(dst_w, dst_h, img_data, 1.0);
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
