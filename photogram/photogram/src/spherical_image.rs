use thunderclap::{CmdDescriptor, CommandArgs, json};

use geo_nd::{Quaternion, Vector};
use ic_photogram::Color8;
use ic_photogram::CylindricalProjection;
use ic_photogram::Idx;
use ic_photogram::{
    CameraInstance, CameraLensProjection, CameraProjection, CameraSensor, LensPolys,
};
use ic_photogram::{Image, ImageDrawable, ImageRgb8};
use ic_photogram::{ImageFileIndex, SphericalImage};
use ic_photogram::{Point2D, Point3D, Quat};

use crate::Result;
use crate::cmd::{CmdArgs, CmdResult};

impl CmdArgs {
    fn lens_poly_of_pts_cmd(&mut self) -> CmdResult {
        let mut sensor_yaws = vec![];
        let mut world_yaws = vec![];
        for i in 0..1000 {
            let s = (i as f64) / 1000.0 * 1.4;
            let dw = self.xy[0][0] * s * s + self.xy[1][0] * s * s * s * s;
            sensor_yaws.push(s);
            world_yaws.push(s * (1.0 + dw));
        }
        let lens_polys = LensPolys::calibration(&sensor_yaws, &world_yaws, 0.2, 60.0, false)?;
        let mut lens = self.camera.lens().clone();
        lens.set_polys(lens_polys.clone());
        self.camera.set_lens(lens);
        Ok(json::to_value(lens_polys)?)
    }

    fn quaternion_mapping_pts_cmd(&mut self) -> CmdResult {
        let q0 = Quat::mapping_vector_pair_to_vector_pair(
            (&self.xyz[0], &self.xyz[1]),
            (&self.xyz[2], &self.xyz[3]),
        );
        let q1 = Quat::mapping_vector_pair_to_vector_pair(
            (&self.xyz[1], &self.xyz[0]),
            (&self.xyz[3], &self.xyz[2]),
        );
        let q = q0.weighted_average_pair(1.0, &q1, 1.0);
        if self.verbose {
            let img0_d0: Point3D = self.xyz[0].into();
            let img0_d1: Point3D = self.xyz[1].into();
            let img1_d0: Point3D = self.xyz[2].into();
            let img1_d1: Point3D = self.xyz[3].into();
            let img0_angle = img0_d0.dot(&img0_d1).acos().to_degrees();
            let img1_angle = img1_d0.dot(&img1_d1).acos().to_degrees();
            eprintln!("Vectors (v0, v1) for img0 subtend {img0_angle} degrees");
            eprintln!("Vectors (v2, v3) for img1 subtend {img1_angle} degrees");
            let img0_d0_mapped = q.apply3(&img0_d0);
            let img0_d1_mapped = q.apply3(&img0_d1);
            eprintln!(
                "Angles (in degrees) between mapped first img dirns and given second img dirns: {:0.6} {:0.6}",
                img0_d0_mapped.dot(&img1_d0).acos().to_degrees(),
                img0_d1_mapped.dot(&img1_d1).acos().to_degrees(),
            );
            eprintln!(
                "Quaternion to map (without accounting for camera) {q} in world dirm {:?}",
                q.conjugate().apply3_arr(&[0., 0., -1.])
            );
        }
        // camera orientation maps world direction vectors to sensor vectors
        let q = q * self.camera.orientation();
        if self.verbose {
            eprintln!(
                "Final orientation {q:?} looking in world dirn {:?}",
                q.conjugate().apply3_arr(&[0., 0., -1.])
            );
        }
        Ok(json::to_value(q)?)
    }

    fn photo_map_pts_cmd(&mut self) -> CmdResult {
        let mut result: Vec<_> = vec![];
        for px_abs_xy in self.xy.iter() {
            let d = self.camera.sensor_px_abs_xy_to_world_dir(*px_abs_xy);
            result.push(d);
        }

        Ok(json::to_value(result)?)
    }

    fn si_delete_image_cmd(&mut self) -> CmdResult {
        if self.spherical_image.is_some() {
            self.spherical_image = None;
        }
        Self::cmd_ok()
    }

    fn si_new_image_cmd(&mut self) -> CmdResult {
        let mut image = SphericalImage::of_shape(self.shape);
        image.set_path_set(self.path_set.clone());
        let image_file = image.add_new_image(self.width, self.height);

        image.add_toplevel_patches(
            image_file,
            self.patch_size,
            0, //patch_subdivision,
        )?;

        let ps: Vec<_> = image.iter_patch_indices().collect();
        fn pix_map(v: Point3D) -> Option<Color8> {
            let r = ((v[0] + 1.) * 127.) as u8;
            let g = ((v[1] + 1.) * 127.) as u8;
            let b = ((v[2] + 1.) * 127.) as u8;
            Some([r, g, b, 0].into())
        }
        for p in ps {
            image.fill_image_patch(0.0, p, &pix_map);
        }
        self.spherical_image = Some(image.into());

        Ok(json::to_value(image_file.opt_index())?)
    }

    fn si_as_json_cmd(&mut self) -> CmdResult {
        self.validate_spherical_image()?;
        let image = self.spherical_image.as_ref().unwrap().borrow();
        let s = {
            if self.pretty_json() {
                serde_json::to_string_pretty(&image.to_desc())?
            } else {
                serde_json::to_string(&image.to_desc())?
            }
        };
        Ok(json::to_value(s)?)
    }

    fn si_add_image_file_cmd(&mut self) -> CmdResult {
        self.validate_spherical_image()?;
        let mut image = self.spherical_image.as_ref().unwrap().borrow_mut();
        let image_file = image.add_new_image(self.width, self.height);
        if let Some(write_filename) = self.write_img() {
            image.set_image_path(image_file, write_filename)?;
        }
        Ok(json::to_value(format!(
            "{}",
            image_file.opt_index().unwrap()
        ))?)
    }

    fn si_set_image_file_filename_cmd(&mut self) -> CmdResult {
        self.validate_spherical_image()?;
        let image_file = ImageFileIndex::from_usize(self.arg_usizes[0]);
        let mut image = self.spherical_image.as_ref().unwrap().borrow_mut();
        if let Some(write_filename) = self.write_img() {
            image.set_image_path(image_file, write_filename)?;
        }
        Self::cmd_ok()
    }

    fn si_write_image_file_cmd(&mut self) -> CmdResult {
        self.validate_spherical_image()?;
        let image = self.spherical_image.as_ref().unwrap().borrow();
        image.write_image(
            ImageFileIndex::from_usize(0), // image_file,
        )?;
        Self::cmd_ok()
    }

    fn si_render_photo_cmd(&mut self) -> CmdResult {
        self.validate_spherical_image()?;
        let image = self.spherical_image.as_ref().unwrap().borrow();
        self.if_verbose(|| eprintln!("Rendering using camera {}", self.camera));

        let (w, h) = self.camera.sensor_px_size();
        let w = w as u32;
        let h = h as u32;
        let mut jpg = ImageRgb8::new(w, h);
        let mut p = Point2D::default();
        for y in 0..h {
            p[1] = y as f64;
            for x in 0..w {
                p[0] = x as f64;
                let d = self.camera.sensor_px_abs_xy_to_world_dir(p);
                if let Some(color) = image.get_pixel_of_direction(&d) {
                    jpg.put(x, y, &color);
                }
            }
        }
        jpg.write(self.write_img().unwrap())?;
        Self::cmd_ok()
    }

    fn si_render_panorama_horizontal(&mut self, jpg: &mut ImageRgb8) -> Result<()> {
        self.validate_spherical_image()?;
        let image = self.spherical_image.as_ref().unwrap().borrow();
        let q = self.camera.orientation();
        self.if_verbose(||
            eprintln!("Rendering panorama of horizontal FOV {} + {} degrees and vertical FOV {} + {} degrees using camera {} {q}",
                self.fov_h,
                self.h_ofs,
                self.fov_v,
                self.v_ofs,
                self.camera));
        let hfov_h = self.fov_h.to_radians() / 2.0;
        let h_ofs = self.h_ofs.to_radians();

        self.cylindrical_lens.set_projection("equirectangular")?;
        self.cylindrical_lens
            .set_vfov(self.fov_v.to_radians(), self.v_ofs.to_radians());
        for y in 0..self.height {
            let y_relative = (y as f64) / (self.height as f64);
            let tan_phi = self.cylindrical_lens.tan_phi_of_y(y_relative);
            for x in 0..self.width {
                // x_relative is in the range +-1.0; lambda is in range h_ofs *- hfov/2
                let x_relative = (x as f64) / (self.width as f64) * 2.0 - 1.0;
                let lambda = x_relative * hfov_h + h_ofs;
                let cos_lambda = lambda.cos();
                let sin_lambda = lambda.sin();
                let p: Point3D = [sin_lambda, -tan_phi, -cos_lambda].into();
                let d = q.apply3(&p.normalize());
                if let Some(color) = image.get_pixel_of_direction(&d) {
                    jpg.put(x, y, &color);
                }
            }
        }
        Ok(())
    }

    fn si_render_panorama_vertical(&mut self, jpg: &mut ImageRgb8) -> Result<()> {
        self.validate_spherical_image()?;
        let image = self.spherical_image.as_ref().unwrap().borrow();
        self.if_verbose(|| eprintln!("Rendering panorama of horizontal FOV {} degrees and vertical FOV {} degrees using camera orientation {}", self.fov_h, self.fov_v, self.camera));
        let q = self.camera.orientation();
        let hfov_v = self.fov_v.to_radians() / 2.0;
        let v_ofs = self.v_ofs.to_radians();

        self.cylindrical_lens
            .set_vfov(self.fov_h.to_radians(), self.h_ofs.to_radians());
        for x in 0..self.width {
            let x_relative = (x as f64) / (self.width as f64);
            let tan_phi = self.cylindrical_lens.tan_phi_of_y(1.0 - x_relative);
            for y in 0..self.height {
                let y_relative = 1.0 - (y as f64) / (self.height as f64) * 2.0;
                let lambda = y_relative * hfov_v + v_ofs;
                let cos_lambda = lambda.cos();
                let sin_lambda = lambda.sin();
                let p: Point3D = [tan_phi, sin_lambda, -cos_lambda].into();
                let d = q.apply3(&p.normalize());
                if let Some(color) = image.get_pixel_of_direction(&d) {
                    jpg.put(x, y, &color);
                }
            }
        }
        Ok(())
    }

    /// Renders a panorama as a cylindrical projection, with y being +-half fov vertical, h being +- half fov horizontal
    ///
    fn si_render_panorama_cmd(&mut self) -> CmdResult {
        let mut jpg = ImageRgb8::new(self.width, self.height);
        if self.render_vertical {
            self.si_render_panorama_vertical(&mut jpg)?;
        } else {
            self.si_render_panorama_horizontal(&mut jpg)?;
            let white = 255_u8.into();
            let black = 0.into();
            if self.x_grid > 0.0 {
                for theta_i in 0..=100 {
                    let color = { if theta_i == 0 { &white } else { &black } };
                    let theta = (theta_i as f64) * self.x_grid;
                    let x = ((theta / self.fov_h) * (self.width as f64)) as u32;
                    if x >= self.width / 2 {
                        break;
                    }
                    for y in 0..self.height {
                        jpg.put(self.width / 2 + x, y, &color);
                        jpg.put(self.width / 2 - x, y, &color);
                    }
                }
            }
            if self.y_grid > 0.0 {
                for phi_i in 0..=100 {
                    let color = { if phi_i == 0 { &white } else { &black } };
                    let phi = (self.v_ofs + (phi_i as f64) * self.y_grid).to_radians();
                    // y = 0.0 -> 0, 1.0 -> height
                    let y = self.cylindrical_lens.y_of_phi(phi) * (self.height as f64);
                    if y < 0.0 || y >= (self.height as f64) {
                        break;
                    }
                    let y = y as u32;
                    for x in 0..self.width {
                        jpg.put(x, y, &color);
                    }
                }
                for phi_i in 1..=100 {
                    let color = { if phi_i == 0 { &white } else { &black } };
                    let phi = (self.v_ofs - (phi_i as f64) * self.y_grid).to_radians();
                    // y = 0.0 -> 0, 1.0 -> height
                    let y = self.cylindrical_lens.y_of_phi(phi) * (self.height as f64);
                    if y < 0.0 || y >= (self.height as f64) {
                        break;
                    }
                    let y = y as u32;
                    for x in 0..self.width {
                        jpg.put(x, y, &color);
                    }
                }
            }
        }
        jpg.write(self.write_img().unwrap())?;
        Self::cmd_ok()
    }

    fn si_read_photo_cmd(&mut self) -> CmdResult {
        self.validate_spherical_image()?;
        let mut image = self.spherical_image.as_ref().unwrap().borrow_mut();
        self.if_verbose(|| eprintln!("Reading image using camera {}", self.camera));

        let jpg = ImageRgb8::read(&self.read_img()[0])?;

        let ps: Vec<_> = image.iter_patch_indices().collect();

        let (w, h) = jpg.size();
        for p in ps {
            image.fill_image_patch(self.blend, p, &|v| {
                Self::pix_map(&self.camera, &jpg, w, h, v)
            });
        }
        Self::cmd_ok()
    }

    fn pix_map<I: ImageDrawable>(
        camera: &CameraInstance,
        src: &I,
        w: u32,
        h: u32,
        v: Point3D,
    ) -> Option<I::Pixel> {
        if let Some(pxy) = camera.world_dir_to_opt_sensor_px_abs_xy(v) {
            if pxy[0] < 0.0 || pxy[0] >= (w as f64) || pxy[1] < 0.0 || pxy[1] >= (h as f64) {
                None
            } else {
                let x = pxy[0] as u32;
                let y = pxy[1] as u32;
                Some(src.get(x, y))
            }
        } else {
            None
        }
    }
    fn si_read_cip_image_cmd(&mut self) -> CmdResult {
        self.validate_spherical_image()?;
        self.validate_cip()?;
        let mut image = self.spherical_image.as_ref().unwrap().borrow_mut();
        let cip = self.cip.as_ref().unwrap().borrow();
        self.if_verbose(|| {
            eprintln!(
                "Reading image {} using camera {}",
                cip.image_filename(),
                cip.camera().borrow()
            )
        });

        let f = self.path_set.find_file_err(&cip.image_filename())?;
        let jpg = ImageRgb8::read(&f)?;

        let ps: Vec<_> = image.iter_patch_indices().collect();

        let (w, h) = jpg.size();
        for p in ps {
            image.fill_image_patch(self.blend, p, &|v| {
                Self::pix_map(&self.camera, &jpg, w, h, v)
            });
        }
        Self::cmd_ok()
    }
}

impl CmdArgs {
    const SI_NEW_CMD: CmdDescriptor<Self> = CmdDescriptor::new("new")
        .about("Create a new spherical image, discarding any other currently held")
        .args(&[
            Self::ARG_SPHERICAL_IMAGE_SHAPE,
            Self::ARG_WIDTH,
            Self::ARG_HEIGHT,
            Self::ARG_PATCH_SIZE,
        ])
        .handler(&Self::si_new_image_cmd);

    const SI_DELETE_CMD: CmdDescriptor<Self> = CmdDescriptor::new("delete")
        .about("Delete the active spherical image; only useful in batch/interactive")
        .args(&[])
        .handler(&Self::si_delete_image_cmd);

    const SI_AS_JSON_CMD: CmdDescriptor<Self> = CmdDescriptor::new("as_json")
        .about("Produce the Json for the spherical image")
        .args(&[])
        .handler(&Self::si_as_json_cmd);

    const SI_ADD_IMAGE_FILE_CMD: CmdDescriptor<Self> = CmdDescriptor::new("add_image_file")
        .about("Add a new image file to the active spherical image, with the given image filename")
        .args(&[Self::ARG_WIDTH, Self::ARG_HEIGHT])
        .handler(&Self::si_add_image_file_cmd);

    const SI_SET_IMAGE_FILE_NAME_CMD: CmdDescriptor<Self> =
        CmdDescriptor::new("set_image_file_filename")
            .about("Set the image file of the active spherical image to a given image filename")
            .args(&[
                Self::ARG_SPHERICAL_IMAGE_FILE_INDEX,
                Self::ARG_WRITE_IMAGE_REQUIRED,
            ])
            .handler(&Self::si_set_image_file_filename_cmd);

    const SI_WRITE_IMAGE_FILE_CMD: CmdDescriptor<Self> = CmdDescriptor::new("write_image_file")
        .about("Write the specified image file to its given image filename")
        .args(&[
            Self::ARG_SPHERICAL_IMAGE_FILE_INDEX,
            Self::ARG_WRITE_IMAGE_OPTIONAL,
        ])
        .handler(&Self::si_write_image_file_cmd);

    const SI_PHOTO_READ_CMD: CmdDescriptor<Self> = CmdDescriptor::new("read_photo")
        .about("Read a photograph and draw it on the image using the given camera description")
        .args(&[
            Self::ARG_USE_ORIENTATION,
            Self::ARG_READ_IMAGE_REQUIRED,
            Self::ARG_BLEND,
        ])
        .handler(&Self::si_read_photo_cmd);

    const SI_READ_CIP_IMAGE_CMD: CmdDescriptor<Self> = CmdDescriptor::new("read_cip_image")
        .about("Read a CIP image using its camera into the spherical image")
        .args(&[Self::ARG_BLEND])
        .handler(&Self::si_read_cip_image_cmd);

    const SI_PHOTO_RENDER_CMD: CmdDescriptor<Self> = CmdDescriptor::new("render_photo")
        .about("Render a photograph as if from a camera etc")
        .args(&[Self::ARG_USE_ORIENTATION, Self::ARG_WRITE_IMAGE_REQUIRED])
        .handler(&Self::si_render_photo_cmd);

    const SI_PANORAMA_RENDER_CMD: CmdDescriptor<Self> = CmdDescriptor::new("render_panorama")
        .about("Render a panorama (cylinder projection) given the camera orientation")
        .args(&[
            Self::ARG_WIDTH,
            Self::ARG_HEIGHT,
            Self::ARG_FOVH,
            Self::ARG_FOVV,
            Self::ARG_H_OFS,
            Self::ARG_V_OFS,
            Self::ARG_X_GRID,
            Self::ARG_Y_GRID,
            Self::ARG_CYLINDRICAL_PROJECTION,
            Self::ARG_USE_ORIENTATION,
            Self::ARG_WRITE_IMAGE_REQUIRED,
        ])
        .handler(&Self::si_render_panorama_cmd);

    // Move to CIP
    const PHOTO_MAP_PTS_CMD: CmdDescriptor<Self> = CmdDescriptor::new("photo_map_pts")
        .about("Map photograph (X,Y) points to world directions")
        .args(&[Self::ARG_ADD_POINT2D])
        .handler(&Self::photo_map_pts_cmd);

    // Move to ?
    const QUAT_MAPPING_PTS_CMD: CmdDescriptor<Self> = CmdDescriptor::new("quaternion_mapping_pts")
        .about(
            "Find orientation mapping two (X,Y,Z) world directions in one photo to two in another",
        )
        .args(&[Self::ARG_ADD_XYZ])
        .handler(&Self::quaternion_mapping_pts_cmd);

    // Move to ?
    const LENS_POLYS_OF_PTS_CMD: CmdDescriptor<Self> = CmdDescriptor::new("lens_polys_of_pts")
        .about("Find lens polynomials mapping the XY as (sensor,world) yaws")
        .args(&[Self::ARG_ADD_XY_LIST])
        .handler(&Self::lens_poly_of_pts_cmd);

    pub(crate) const SPHERICAL_IMAGE_CMD: CmdDescriptor<Self> =
        CmdDescriptor::new("spherical_image")
            .about("Spherical image processor")
            .args(&[])
            .cmds(&[
                Self::SI_NEW_CMD,
                Self::SI_DELETE_CMD,
                Self::SI_AS_JSON_CMD,
                Self::SI_ADD_IMAGE_FILE_CMD,
                Self::SI_SET_IMAGE_FILE_NAME_CMD,
                Self::SI_WRITE_IMAGE_FILE_CMD,
                Self::QUAT_MAPPING_PTS_CMD,
                Self::LENS_POLYS_OF_PTS_CMD,
                Self::PHOTO_MAP_PTS_CMD,
                Self::SI_PHOTO_RENDER_CMD,
                Self::SI_PHOTO_READ_CMD,
                Self::SI_READ_CIP_IMAGE_CMD,
                Self::SI_PANORAMA_RENDER_CMD,
            ]);
}
