use std::collections::HashMap;

use thunderclap::{
    ArgCount, ArgDescriptor, CmdDescriptor, CmdProperty, CommandArgs, CommandBuilder,
};

use geo_nd::{Quaternion, Vector};
use ic_base::{JsonParsable, PathSet, Point2D, Point3D, Quat, QuaternionDesc, Result};
use ic_camera::{CameraDatabase, CameraInstance, CameraInstanceDesc, CameraProjection, LensPolys};
use ic_image::{Image, ImageDrawable, ImageRgb8};
use ic_spherical_image::{ImageFileIndex, SphericalImage, SphericalImageShape};
use indexed::Idx;
use star_catalog::Catalog as StarCatalog;

#[derive(Default)]
pub struct SphericalImageCommand {
    verbose: bool,
    pretty_json: bool,

    width: u32,
    height: u32,
    patch_size: u32,

    fov_h: f64,
    fov_v: f64,

    active_image_name: Option<String>,
    shape: SphericalImageShape,
    file_path_set: PathSet,

    cdb: CameraDatabase,
    camera: CameraInstance,

    images: HashMap<String, SphericalImage<ImageRgb8>>,
    blend: f64,

    star_magnitude: f32,
    star_catalog: Option<StarCatalog>,

    // These are reset before the command
    xy: Vec<Point2D>,
    xyz: Vec<Point3D>,
    write_filename: Option<String>,
    read_filename: Option<String>,

    // Positional string / f64 / usize arguments
    arg_strings: Vec<String>,
    arg_f64s: Vec<f64>,
    arg_usizes: Vec<usize>,
}

impl std::fmt::Debug for SphericalImageCommand {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "CmdArgs{{")?;
        if self.verbose {
            write!(fmt, "verbose, ")?;
        }
        if self.pretty_json {
            write!(fmt, "pretty_json")?;
        }
        Ok(())
    }
}

impl CommandArgs for SphericalImageCommand {
    type Error = ic_base::Error;
    type Value = String;
    const PROPERTIES: &[thunderclap::CmdProperty<'static, Self, Self::Value, Self::Error>] =
        &[CmdProperty {
            name: "orientation",
            get_fn: &|cmd_args| serde_json::to_string(&cmd_args.camera.orientation()).ok(),
            set_value_fn: &|cmd_args, s| {
                QuaternionDesc::load_json(s, &()).map(|q| {
                    cmd_args.camera.set_orientation(&q);
                    true
                })
            },
        }];
    fn cmd_ok() -> std::result::Result<Self::Value, Self::Error> {
        Ok("".into())
    }
    fn value_from_str(s: &str) -> std::result::Result<Self::Value, Self::Error> {
        Ok(s.into())
    }
    fn reset_args(&mut self) {
        self.write_filename = None;
        self.read_filename = None;
        self.xy.clear();
        self.xyz.clear();
        self.blend = 0.0;
    }
}

impl SphericalImageCommand {
    fn if_verbose<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        if self.verbose {
            f()
        }
    }

    const ARG_VERBOSE: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_flag(
        "verbose",
        Some('v'),
        "Enable verbose output",
        &SphericalImageCommand::set_verbose,
    );
    fn set_verbose(&mut self, verbose: bool) -> Result<()> {
        self.verbose = verbose;
        Ok(())
    }
    pub fn verbose(&self) -> bool {
        self.verbose
    }

    const ARG_PRETTY_JSON: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_flag(
        "pretty_json",
        None,
        "Use pretty-printing for Json output",
        &SphericalImageCommand::set_pretty_json,
    );
    fn set_pretty_json(&mut self, pretty_json: bool) -> Result<()> {
        self.pretty_json = pretty_json;
        Ok(())
    }
    pub fn pretty_json(&self) -> bool {
        self.pretty_json
    }

    const ARG_WIDTH: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_u32(
        "width",
        None,
        "Set the width for the operation",
        ArgCount::Required,
        None,
        &SphericalImageCommand::set_width,
    );
    fn set_width(&mut self, v: u32) -> Result<()> {
        self.width = v;
        Ok(())
    }
    const ARG_HEIGHT: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_u32(
        "height",
        None,
        "Set the height for the operation",
        ArgCount::Required,
        None,
        &SphericalImageCommand::set_height,
    );
    fn set_height(&mut self, v: u32) -> Result<()> {
        self.height = v;
        Ok(())
    }
    const ARG_PATCH_SIZE: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_u32(
        "patch_size",
        Some('p'),
        "Set the patch size for the operation",
        ArgCount::Required,
        None,
        &SphericalImageCommand::set_patch_size,
    );
    fn set_patch_size(&mut self, v: u32) -> Result<()> {
        self.patch_size = v;
        Ok(())
    }

    const ARG_FOVH: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_f64(
        "fovh",
        None,
        "Set the hoizontal FOV to use, in degrees",
        ArgCount::Required,
        None,
        &|s: &mut SphericalImageCommand, v: f64| {
            s.fov_h = v;
            Ok(())
        },
    );
    const ARG_FOVV: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_f64(
        "fovv",
        None,
        "Set the vertical FOV to use, in degrees",
        ArgCount::Required,
        None,
        &|s: &mut SphericalImageCommand, v: f64| {
            s.fov_v = v;
            Ok(())
        },
    );

    const ARG_ADD_XY: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_string(
        "xy",
        None,
        "Provide a list of (X,Y) coordinates",
        ArgCount::Required,
        None,
        &SphericalImageCommand::add_xy,
    );
    fn add_xy(&mut self, v: &str) -> Result<()> {
        self.xy
            .extend_from_slice(&Vec::<Point2D>::load_json(v, &())?);
        Ok(())
    }
    const ARG_ADD_XYZ: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_string(
        "xyz",
        None,
        "Provide a list of (X,Y,Z) coordinates",
        ArgCount::Required,
        None,
        &SphericalImageCommand::add_xyz,
    );
    fn add_xyz(&mut self, v: &str) -> Result<()> {
        self.xyz
            .extend_from_slice(&Vec::<Point3D>::load_json(v, &())?);
        Ok(())
    }

    const ARG_STAR_MAGNITUDE: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_f32(
        "star_magnitude",
        None,
        "Set the star magnitude, using a default value of 8.0",
        ArgCount::Required,
        Some("8.0"),
        &SphericalImageCommand::set_star_magnitude,
    );
    fn set_star_magnitude(&mut self, v: f32) -> Result<()> {
        self.star_magnitude = v;
        Ok(())
    }

    const ARG_STAR_CATALOG: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_string(
        "star_catalog",
        None,
        "Set the star catalog, used fo generate sky map images of the stars. Ensure the star magnitude is set before using this option",
        ArgCount::Optional,
        None,
        &SphericalImageCommand::set_star_catalog,
    );
    fn set_star_catalog(&mut self, filename: &str) -> Result<()> {
        let mut catalog = StarCatalog::load_catalog(filename, self.star_magnitude)?;
        catalog.derive_data();
        self.star_catalog = Some(catalog);
        Ok(())
    }

    const ARG_CLEAR_FILE_PATH: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_flag(
        "clear_file_path",
        None,
        "Clear the file path",
        &SphericalImageCommand::clear_file_path,
    );
    const ARG_ADD_FILE_PATH: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_string(
        "file_path",
        Some('P'),
        "Add a file path to the path set",
        ArgCount::Any,
        None,
        &SphericalImageCommand::add_file_path,
    );

    fn clear_file_path(&mut self, _s: bool) -> Result<()> {
        self.file_path_set.clear();
        Ok(())
    }
    fn add_file_path(&mut self, s: &str) -> Result<()> {
        self.file_path_set.add_path(s)?;
        Ok(())
    }

    const ARG_SET_WRITE_FILE: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_string(
        "write",
        Some('W'),
        "Set the filename to write to",
        ArgCount::Optional,
        None,
        &SphericalImageCommand::set_write_filename,
    );
    const ARG_SET_WRITE_FILE_REQUIRED: ArgDescriptor<SphericalImageCommand> =
        ArgDescriptor::arg_string(
            "write",
            Some('W'),
            "Set the filename to write to",
            ArgCount::Required,
            None,
            &SphericalImageCommand::set_write_filename,
        );
    fn set_write_filename(&mut self, filename: &str) -> Result<()> {
        self.write_filename = Some(filename.into());
        Ok(())
    }
    pub fn write_filename(&self) -> Option<&str> {
        self.write_filename.as_deref()
    }

    const ARG_SET_READ_FILE_REQUIRED: ArgDescriptor<SphericalImageCommand> =
        ArgDescriptor::arg_string(
            "read",
            Some('r'),
            "Set an image filename to read",
            ArgCount::Required,
            None,
            &SphericalImageCommand::set_read_filename,
        );
    fn set_read_filename(&mut self, filename: &str) -> Result<()> {
        self.read_filename = Some(filename.into());
        Ok(())
    }
    pub fn read_filename(&self) -> Option<&str> {
        self.read_filename.as_deref()
    }

    const ARG_SET_SHAPE: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_string(
        "shape",
        None,
        "Set the toplevel shape",
        ArgCount::Optional,
        None,
        &SphericalImageCommand::set_shape,
    );
    fn set_shape(&mut self, shape_name: &str) -> Result<()> {
        self.shape = shape_name.parse::<SphericalImageShape>()?;
        Ok(())
    }
    pub fn shape(&self) -> SphericalImageShape {
        self.shape
    }

    const ARG_SET_NAME: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_string(
        "name",
        Some('n'),
        "Set the active spherical image name",
        ArgCount::Optional,
        None,
        &SphericalImageCommand::set_active_image_name,
    );
    const ARG_SET_NAME_REQUIRED: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_string(
        "name",
        Some('n'),
        "Set the active spherical image name",
        ArgCount::Required,
        None,
        &SphericalImageCommand::set_active_image_name,
    );
    fn set_active_image_name(&mut self, name: &str) -> Result<()> {
        self.active_image_name = Some(name.to_owned());
        Ok(())
    }

    const ARG_CAMERA_DATABASE: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_string(
        "camera_db",
        None,
        "Camera database JSON filename",
        ArgCount::Optional,
        None,
        &SphericalImageCommand::set_camera_db,
    );
    const ARG_CAMERA: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_string(
        "camera",
        Some('c'),
        "Camera lens, placement and orientation JSON",
        ArgCount::Optional,
        None,
        &SphericalImageCommand::set_camera_file,
    );
    const ARG_BODY: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_string(
        "use_body",
        None,
        "Select a camera body from the database",
        ArgCount::Optional,
        None,
        &SphericalImageCommand::set_camera_body,
    );
    const ARG_LENS: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_string(
        "use_lens",
        None,
        "Select a camera lens from the database",
        ArgCount::Optional,
        None,
        &SphericalImageCommand::set_camera_lens,
    );
    const ARG_FOCUS_DISTANCE: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_f64(
        "use_focus",
        None,
        "Set a focus distance in mm",
        ArgCount::Optional,
        None,
        &SphericalImageCommand::set_camera_focus_distance,
    );
    const ARG_LOOK_AT: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_string(
        "orientation",
        None,
        "Direction camera is pointing, as Json QuaternionDesc",
        ArgCount::Optional,
        None,
        &SphericalImageCommand::set_orientation,
    );
    const ARG_POLY_JSON: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_string(
        "use_poly_json",
        None,
        "Specify the JSON for the lens polynomials in the camera",
        ArgCount::Optional,
        None,
        &SphericalImageCommand::set_polys_json,
    );

    fn set_camera_db(&mut self, filename: &str) -> Result<()> {
        let (cdb_filename, camera_db) =
            CameraDatabase::load_json_file(&self.file_path_set, filename, &())?;
        self.if_verbose(|| eprintln!("Loaded camera database from '{cdb_filename}'"));
        self.cdb = camera_db;
        Ok(())
    }
    fn set_camera_file(&mut self, filename: &str) -> Result<()> {
        let (_, camera) =
            CameraInstanceDesc::load_json_file(&self.file_path_set, filename, &self.cdb)?;
        self.camera = camera;
        Ok(())
    }
    fn set_camera_body(&mut self, name: &str) -> Result<()> {
        self.camera.set_body(self.cdb.get_body_err(name)?.clone());
        Ok(())
    }
    fn set_camera_lens(&mut self, name: &str) -> Result<()> {
        self.camera.set_lens(self.cdb.get_lens_err(name)?.clone());
        Ok(())
    }
    fn set_camera_focus_distance(&mut self, v: f64) -> Result<()> {
        self.camera.set_focus_distance(v);
        Ok(())
    }
    fn set_orientation(&mut self, json: &str) -> Result<()> {
        self.camera
            .set_orientation(&QuaternionDesc::load_json(json, &())?);
        Ok(())
    }
    fn set_polys_json(&mut self, polys: &str) -> Result<()> {
        let lens_polys = LensPolys::load_json(polys, &())?;
        let mut lens = self.camera.lens().clone();
        lens.set_polys(lens_polys);
        self.camera.set_lens(lens);
        Ok(())
    }

    const ARG_SET_BLEND: ArgDescriptor<SphericalImageCommand> = ArgDescriptor::arg_f64(
        "blend",
        None,
        "Amount of the current image to use compared to the new image",
        ArgCount::Optional,
        None,
        &|m: &mut SphericalImageCommand, v: f64| {
            m.blend = v;
            Ok(())
        },
    );

    const NEW_IMAGE_CMD: CmdDescriptor<Self> = CmdDescriptor::new("new_image")
        .about("Create a new spherical image")
        .args(&[Self::ARG_SET_NAME_REQUIRED, Self::ARG_SET_SHAPE])
        .handler(&Self::new_image_cmd);
    const JSON_IMAGE_CMD: CmdDescriptor<Self> = CmdDescriptor::new("json_image")
        .about("Produce the Json for the spherical image")
        .args(&[Self::ARG_SET_NAME])
        .handler(&Self::json_image_cmd);

    const ADD_IMAGE_FILE_CMD: CmdDescriptor<Self> = CmdDescriptor::new("add_image_file")
        .about("Add a new image file to the active spherical image, with the given image filename")
        .args(&[
            Self::ARG_SET_NAME,
            Self::ARG_WIDTH,
            Self::ARG_HEIGHT,
            Self::ARG_SET_WRITE_FILE,
        ])
        .handler(&Self::add_image_file_cmd);

    const WRITE_IMAGE_FILE_CMD: CmdDescriptor<Self> = CmdDescriptor::new("write_image_file")
        .about("Write the specified image file to its given image filename")
        .args(&[])
        .handler(&Self::write_image_file);

    const ADD_TOPLVEL_PATCHES_CMD: CmdDescriptor<Self> = CmdDescriptor::new("add_toplevel_patches")
        .about("Adds toplevel patches to a spherical image with *no* patches")
        .args(&[Self::ARG_SET_NAME, Self::ARG_PATCH_SIZE])
        .handler(&Self::add_toplevel_patches_cmd);

    const DELETE_IMAGE_CMD: CmdDescriptor<Self> = CmdDescriptor::new("delete_image")
        .about("Delete the active spherical image; only useful in batch/interactive")
        .args(&[Self::ARG_SET_NAME_REQUIRED])
        .handler(&Self::delete_image_cmd);

    const PHOTO_READ_CMD: CmdDescriptor<Self> = CmdDescriptor::new("read_photo")
        .about("Read a photograph and draw it on the image using the given camera description")
        .args(&[
            Self::ARG_LOOK_AT,
            Self::ARG_SET_READ_FILE_REQUIRED,
            Self::ARG_SET_BLEND,
        ])
        .handler(&Self::read_photo_cmd);

    const PHOTO_RENDER_CMD: CmdDescriptor<Self> = CmdDescriptor::new("render_photo")
        .about("Render a photograph as if from a camera etc")
        .args(&[Self::ARG_LOOK_AT, Self::ARG_SET_WRITE_FILE_REQUIRED])
        .handler(&Self::render_photo_cmd);

    const PANORAMA_RENDER_CMD: CmdDescriptor<Self> = CmdDescriptor::new("render_panorama")
        .about("Render a panorama (cylinder projection) given the camera orientation")
        .args(&[
            Self::ARG_WIDTH,
            Self::ARG_HEIGHT,
            Self::ARG_FOVH,
            Self::ARG_FOVV,
            Self::ARG_LOOK_AT,
            Self::ARG_SET_WRITE_FILE_REQUIRED,
        ])
        .handler(&Self::render_panorama_cmd);

    const PHOTO_MAP_PTS_CMD: CmdDescriptor<Self> = CmdDescriptor::new("photo_map_pts")
        .about("Map photograph (X,Y) points to world directions")
        .args(&[Self::ARG_ADD_XY])
        .handler(&Self::photo_map_pt_cmd);

    const QUAT_MAPPING_PTS_CMD: CmdDescriptor<Self> = CmdDescriptor::new("quaternion_mapping_pts")
        .about(
            "Find orientation mapping two (X,Y,Z) world directions in one photo to two in another",
        )
        .args(&[Self::ARG_ADD_XYZ])
        .handler(&Self::quaternion_mapping_pts_cmd);

    const LENS_POLYS_OF_PTS_CMD: CmdDescriptor<Self> = CmdDescriptor::new("lens_polys_of_pts")
        .about("Find lens polynomials mapping the XY as (sensor,world) yaws")
        .args(&[Self::ARG_ADD_XY])
        .handler(&Self::lens_poly_of_pts_cmd);

    const BASE_CMD: CmdDescriptor<Self> = CmdDescriptor::new("spherical_image")
        .about("Spherical image processor")
        .version("0.1.0")
        .args(&[
            Self::ARG_VERBOSE,
            Self::ARG_PRETTY_JSON,
            Self::ARG_CLEAR_FILE_PATH,
            Self::ARG_ADD_FILE_PATH,
            Self::ARG_CAMERA_DATABASE,
            Self::ARG_CAMERA,
            Self::ARG_BODY,
            Self::ARG_LENS,
            Self::ARG_POLY_JSON,
            Self::ARG_FOCUS_DISTANCE,
            Self::ARG_LOOK_AT,
        ])
        .cmds(&[
            Self::NEW_IMAGE_CMD,
            Self::JSON_IMAGE_CMD,
            Self::ADD_IMAGE_FILE_CMD,
            Self::WRITE_IMAGE_FILE_CMD,
            Self::ADD_TOPLVEL_PATCHES_CMD,
            Self::DELETE_IMAGE_CMD,
            Self::QUAT_MAPPING_PTS_CMD,
            Self::LENS_POLYS_OF_PTS_CMD,
            Self::PHOTO_MAP_PTS_CMD,
            Self::PHOTO_RENDER_CMD,
            Self::PHOTO_READ_CMD,
            Self::PANORAMA_RENDER_CMD,
        ]);

    pub fn command_builder() -> CommandBuilder<Self> {
        Self::BASE_CMD.build()
    }

    fn add_string_arg(&mut self, s: &str) -> Result<()> {
        self.arg_strings.push(s.to_owned());
        Ok(())
    }

    fn add_f64_arg(&mut self, v: f64) -> Result<()> {
        self.arg_f64s.push(v);
        Ok(())
    }

    fn add_usize_arg(&mut self, v: usize) -> Result<()> {
        self.arg_usizes.push(v);
        Ok(())
    }

    fn validate_active_image_name(&self) -> Result<()> {
        if let Some(name) = self.active_image_name.as_deref() {
            if self.images.contains_key(name) {
                Ok(())
            } else {
                Err(format!("Active image {name} not found").into())
            }
        } else {
            Err("No active image name set".into())
        }
    }

    fn lens_poly_of_pts_cmd(&mut self) -> Result<String> {
        let mut sensor_yaws = vec![];
        let mut world_yaws = vec![];
        for i in 0..1000 {
            let s = (i as f64) / 1000.0 * 1.4;
            let dw = self.xy[0][0] * s * s + self.xy[1][0] * s * s * s * s;
            sensor_yaws.push(s);
            world_yaws.push(s * (1.0 + dw));
        }
        let lens_polys = LensPolys::calibration(8, &sensor_yaws, &world_yaws, 0.0, 10000.0, false)?;
        let mut lens = self.camera.lens().clone();
        lens.set_polys(lens_polys.clone());
        self.camera.set_lens(lens);
        Ok(serde_json::to_string(&lens_polys)?)
    }
    fn quaternion_mapping_pts_cmd(&mut self) -> Result<String> {
        let q = Quat::mapping_vector_pair_to_vector_pair(
            (&self.xyz[0], &self.xyz[1]),
            (&self.xyz[2], &self.xyz[3]),
        );
        let img0_d0: Point3D = self.xyz[0].into();
        let img0_d1: Point3D = self.xyz[1].into();
        let img1_d0: Point3D = self.xyz[2].into();
        let img1_d1: Point3D = self.xyz[3].into();
        let img0_d0_mapped = q.apply3(&img0_d0);
        let img0_d1_mapped = q.apply3(&img0_d1);
        eprintln!(
            "Distances of mapped pts: {:0.6} {:0.6}",
            img0_d0_mapped.distance(&img1_d0),
            img0_d1_mapped.distance(&img1_d1),
        );
        let q = q * self.camera.orientation();
        Ok(serde_json::to_string(&q)?)
    }
    fn photo_map_pt_cmd(&mut self) -> Result<String> {
        let result: Vec<_> = self
            .xy
            .iter()
            .map(|xy| {
                self.camera
                    .camera_txty_to_world_dir(&self.camera.px_abs_xy_to_camera_txty(xy))
            })
            .collect();

        Ok(serde_json::to_string(&result)?)
    }

    fn delete_image_cmd(&mut self) -> Result<String> {
        self.validate_active_image_name()?;
        if let Some(name) = self.active_image_name.as_deref() {
            self.images.remove(name);
        }
        Self::cmd_ok()
    }

    fn new_image_cmd(&mut self) -> Result<String> {
        if let Some(name) = self.active_image_name.as_deref() {
            if self.images.contains_key(name) {
                return Err(format!(
                    "Spherical image {name} is already present - cannot create it again"
                )
                .into());
            }
            let mut image = SphericalImage::of_shape(self.shape);
            image.set_path_set(self.file_path_set.clone());
            self.images.insert(name.to_owned(), image);
        }

        Self::cmd_ok()
    }

    fn json_image_cmd(&mut self) -> Result<String> {
        self.validate_active_image_name()?;
        let pretty_json = self.pretty_json();
        let name = self.active_image_name.as_deref().unwrap();
        let image = self.images.get_mut(name).unwrap();
        if pretty_json {
            Ok(serde_json::to_string_pretty(&image.to_desc())?)
        } else {
            Ok(serde_json::to_string(&image.to_desc())?)
        }
    }

    fn add_image_file_cmd(&mut self) -> Result<String> {
        self.validate_active_image_name()?;
        let name = self.active_image_name.as_deref().unwrap();
        let image = self.images.get_mut(name).unwrap();
        let image_file = image.add_new_image(self.width, self.height);
        if let Some(write_filename) = self.write_filename.as_deref() {
            image.set_image_path(image_file, write_filename);
        }
        Ok(format!("{}", image_file.opt_index().unwrap()))
    }

    fn set_image_file_filename(&mut self) -> Result<String> {
        self.validate_active_image_name()?;
        let name = self.active_image_name.as_deref().unwrap();
        self.images.get_mut(name).unwrap().add_toplevel_patches(
            ImageFileIndex::from_usize(0), // image_file,
            self.patch_size,
            0, //patch_subdivision,
        )?;
        Self::cmd_ok()
    }

    fn add_toplevel_patches_cmd(&mut self) -> Result<String> {
        self.validate_active_image_name()?;
        let name = self.active_image_name.as_deref().unwrap();
        let image = self.images.get_mut(name).unwrap();
        image.add_toplevel_patches(
            ImageFileIndex::from_usize(0), // image_file,
            self.patch_size,
            0, //patch_subdivision,
        )?;

        let ps: Vec<_> = image.iter_patch_indices().collect();
        fn pix_map(v: Point3D) -> Option<ic_image::Color8> {
            let r = ((v[0] + 1.) * 127.) as u8;
            let g = ((v[1] + 1.) * 127.) as u8;
            let b = ((v[2] + 1.) * 127.) as u8;
            Some([r, g, b, 0].into())
        }
        for p in ps {
            image.fill_image_patch(0.0, p, &pix_map);
        }

        Self::cmd_ok()
    }

    fn write_image_file(&mut self) -> Result<String> {
        self.validate_active_image_name()?;
        let name = self.active_image_name.as_deref().unwrap();
        self.images.get(name).unwrap().write_image(
            ImageFileIndex::from_usize(0), // image_file,
        )?;
        Self::cmd_ok()
    }

    fn render_photo_cmd(&mut self) -> Result<String> {
        self.validate_active_image_name()?;
        let name = self.active_image_name.as_deref().unwrap();
        let image = self.images.get(name).unwrap();
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
                let txty = self.camera.px_abs_xy_to_camera_txty(&p);
                let d = self.camera.camera_txty_to_world_dir(&txty);
                if let Some(color) = image.get_pixel_of_direction(&d) {
                    jpg.put(x, y, &color);
                }
            }
        }
        jpg.write(self.write_filename().unwrap())?;
        Self::cmd_ok()
    }

    fn render_panorama_cmd(&mut self) -> Result<String> {
        self.validate_active_image_name()?;
        let name = self.active_image_name.as_deref().unwrap();
        let image = self.images.get(name).unwrap();
        self.if_verbose(|| eprintln!("Rendering panorama of horizontal FOV {} degrees and vertical FOV {} degrees using camera orientation {}", self.fov_h, self.fov_v, self.camera));

        // half field-of-view vertically
        let hfov_v = self.fov_v.to_radians() / 2.0;
        let hfov_h = self.fov_h.to_radians() / 2.0;

        let mut jpg = ImageRgb8::new(self.width, self.height);
        let q = self.camera.orientation(); // .conjugate();
        for y in 0..self.height {
            let phi = ((y as f64) / ((self.height - 1) as f64) * 2.0 - 1.0) * hfov_v;
            let cos_phi = phi.cos();
            let sin_phi = phi.sin();
            for x in 0..self.width {
                let theta = (((x as f64) / (self.width - 1) as f64) * 2.0 - 1.0) * hfov_h;
                let cos_theta = theta.cos();
                let sin_theta = theta.sin();
                let p: Point3D = [sin_theta * cos_phi, -sin_phi, cos_theta * cos_phi].into();
                let d = q.apply3(&p);
                if let Some(color) = image.get_pixel_of_direction(&d) {
                    jpg.put(x, y, &color);
                }
            }
        }
        jpg.write(self.write_filename().unwrap())?;
        Self::cmd_ok()
    }

    fn read_photo_cmd(&mut self) -> Result<String> {
        self.validate_active_image_name()?;
        self.if_verbose(|| eprintln!("Reading image using camera {}", self.camera));
        let name = self.active_image_name.as_deref().unwrap();
        let image = self.images.get_mut(name).unwrap();

        let jpg = ImageRgb8::read(self.read_filename.as_deref().unwrap())?;

        let ps: Vec<_> = image.iter_patch_indices().collect();
        fn pix_map(
            camera: &CameraInstance,
            src: &ImageRgb8,
            w: u32,
            h: u32,
            v: Point3D,
        ) -> Option<ic_image::Color8> {
            if let Some(pxy) = camera.world_dir_to_opt_px_abs_xy(&v) {
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
        let (w, h) = jpg.size();
        for p in ps {
            image.fill_image_patch(self.blend, p, &|v| pix_map(&self.camera, &jpg, w, h, v));
        }
        Self::cmd_ok()
    }
}
