//a Documentation
//! Test of camera calibration from stars (IMG_4924.JPG)
//!
//! The stars were captured on a Canon Rebel T2i, with a 50mm lens focused on 'infinity'
//!
//! The camera is face-on to the grid (which is graph paper); the
//! approximate interscetions of 550 grid lines was capture as sensor pixel
//! coordinates XY and mm XY pairings. The grid is assumed to be at Z=0.
//!
//! Some of the pairings (given by pt_indices) are used to create a
//! ModelLineSet, which is a set of ModelLines (between grid points,
//! hence all in the plane Z=0) and the angles subtended by the
//! camera, given by the sensor pixel coordinates through the
//! camera/lens model from the database (i.e. this includes the lens
//! mapping)
//!
//! Note that this does not assume the orientation of position of the
//! camera; it purely uses the absolute pixel XY to relative pixel XY
//! to TanXTanY to RollYaw through the lens mapping to a new RollYaw
//! to a TanXTanY in model space to a unit direction vector.
//!
//! From this ModelLineSet a position in space is determined, using
//! the 'million possible points on a ModelLinetSubtended surface)
//! approach.
//!
//! This camera position is then optimized further by delta
//! adjustments in the ModelLineSet.
//!
//! From this 'known good position' the best orientation can be
//! determined, by creating quaternion orientations for every pair of
//! pairings in the pt_indices by:
//!
//!   1. Find the unit direction from the camera to both of the model points (A, B)
//!
//!   2. Find the the unit direction for the camera on it sensor (from the pairing)
//!
//!   3. Generate a quaternion qm that rotates model point direction A to the vector (0,0,1)
//!
//!   4. Generate a quaternion qc that rotates camera point direction A to the vector (0,0,1)
//!
//!   5. Apply qm to model point direction B to yield dj_m
//!
//!   6. Apply qc' to camera point direction B to yield dj_c
//!
//!   7. Note that dj_m and dj_c should have the same *yaw* but a different *roll*
//!
//!   8. Determine the roll required to map dj_m to dj_c
//!
//!   9. Generate quaternion qz which is the rotation around Z for the roll
//!
//!   10. Generate quaternion q = qm.qz.qc
//!
//!   11. Note that q transforms model point direction A to camera point direction A
//!
//!   12. Note that q transforms model point direction B to camera point direction B (if the yaws were identical)
//!
//!   13. Note hence that q is the orientation of a camera that matches the view of model points A and B
//!
//! The value 'q' is inaccurate if the *yaw* values are different -
//! i.e. if the angle subtended by the line between the two points on
//! the camera does not match the angle subtended by the line between
//! the two points in model space as seen by the camera at its given location.
//!
//! The value of 'q' for *every* pair of pairings (A to B, and also B
//! to A) is generated, and an average of these quaternions is used as
//! the orientation of the camera
//!
//! Given the position and orientation of the camera the unit
//! direction vector to every model point from the camera can be
//! determined, and converted to a *roll* and *yaw*. The corresponding
//! camera sensor direction (potentially without going through the lens mapping)
//! can be determined, and presented also as a *roll* and *yaw*.
//!
//! A graph of camera yaw versus model yaw can be produced; if no lens
//! mapping had been used the this should be approximately a single
//! curve that is the polynomial for the lens (mapping yaw in camera
//! to yaw in model space).
//!
//! However, if the *centre* of the camera (upon which the absolute
//! camera sensor XY to camera unit direction vectors depend) has an
//! incorrect value (is the lens centred on the mid-point of the
//! sensor?) then the curve for the yaw-yaw for the camera points in
//! the upper right quadrant of the sensor will have approximately the
//! same shape, but will have a differentoffset, to that from the
//! lower right quadrant.
//!
//! So here we plot *four* graphs, one for each quadrant.
//!
//! For *all* of the points together a pair of polynomials (one
//! camera-to-model, the other the inverse) are generated
//!
//! The process to calibrate the camera is thus to:
//!
//!  1. Reset its lens mapping polynomial
//!
//!  2. Reset the centre of the lens (to the middle of the sensor)
//!
//!  3. Run the program and inspect the graphs
//!
//!  4. Adjust the centre of the sensor if the four graphs are
//!  noticeable offset from each other; repeat from step 3
//!
//!  5. Once the graphs are all deemed reasonable, copy the
//!  polynomials calculated in to the lens mapping.
//!
//!  6. Rerun, and the graphs should be near identity, and the
//!  calibration is complete.
//!  

//a Imports

use std::collections::HashMap;
use std::rc::Rc;

use geo_nd::{quat, Quaternion, Vector};
use serde::{Deserialize, Serialize};
use star_catalog::{hipparcos, Catalog, CatalogIndex, Subcube};

use ic_base::json;
use ic_base::Quat;
use ic_base::{Point2D, Point3D, Result, RollYaw, TanXTanY};
use ic_camera::polynomial;
use ic_camera::polynomial::CalcPoly;
use ic_camera::{serialize_body_name, serialize_lens_name};
use ic_camera::{CameraBody, CameraLens, CameraPolynomial, CameraPolynomialDesc};
use ic_camera::{CameraDatabase, CameraInstance, CameraProjection};
use ic_image::{Color, Image, ImageRgb8};

use ic_mapping::{ModelLineSet, NamedPoint, NamedPointSet, PointMappingSet};

use ic_cmdline::builder::{ArgFn, CommandArgs, CommandBuilder, CommandFn, CommandSet};

#[derive(Default)]
pub struct CmdArgs {
    cdb: Option<CameraDatabase>,
    cal: Option<StarCalibrate>,
    read_img: Option<String>,
    write_img: Option<String>,
}
impl CommandArgs for CmdArgs {
    type Error = ic_base::Error;
    type Value = ();
}
fn arg_camera_db(args: &mut CmdArgs, matches: &clap::ArgMatches) -> Result<()> {
    Ok(())
}

//a Useful functions
//fi orientation_mapping
/// Get q which maps model to camera
///
/// dc === quat::apply3(q, dm)
fn orientation_mapping_triangle(
    di_m: &[f64; 3],
    dj_m: &[f64; 3],
    dk_m: &[f64; 3],
    di_c: Point3D,
    dj_c: Point3D,
    dk_c: Point3D,
) -> Quat {
    let mut qs = vec![];
    qs.push((1.0, orientation_mapping(di_m, dj_m, di_c, dj_c).into()));
    qs.push((1.0, orientation_mapping(di_m, dk_m, di_c, dk_c).into()));
    qs.push((1.0, orientation_mapping(dj_m, dk_m, dj_c, dk_c).into()));
    qs.push((1.0, orientation_mapping(dj_m, di_m, dj_c, di_c).into()));
    qs.push((1.0, orientation_mapping(dk_m, di_m, dk_c, di_c).into()));
    qs.push((1.0, orientation_mapping(dk_m, dj_m, dk_c, dj_c).into()));
    quat::weighted_average_many(qs.iter().copied()).into()
}
fn orientation_mapping(di_m: &[f64; 3], dj_m: &[f64; 3], di_c: Point3D, dj_c: Point3D) -> Quat {
    let z_axis: Point3D = [0., 0., 1.].into();
    let qi_c: Quat = quat::rotation_of_vec_to_vec(&di_c.into(), &z_axis.into()).into();
    let qi_m: Quat = quat::rotation_of_vec_to_vec(di_m, &z_axis.into()).into();

    let dj_c_rotated: Point3D = quat::apply3(qi_c.as_ref(), dj_c.as_ref()).into();
    let dj_m_rotated: Point3D = quat::apply3(qi_m.as_ref(), dj_m).into();

    let theta_dj_m = dj_m_rotated[0].atan2(dj_m_rotated[1]);
    let theta_dj_c = dj_c_rotated[0].atan2(dj_c_rotated[1]);
    let theta = theta_dj_m - theta_dj_c;
    let theta_div_2 = theta / 2.0;
    let cos_2theta = theta_div_2.cos();
    let sin_2theta = theta_div_2.sin();
    let q_z = Quat::of_rijk(cos_2theta, 0.0, 0.0, sin_2theta);

    qi_c.conjugate() * q_z * qi_m
}
fn closest_star(catalog: &Catalog, v: Point3D) -> Option<(f64, CatalogIndex)> {
    let s = Subcube::of_vector(&v.into_array().into());
    let mut closest = None;
    for s in s.iter_range(2) {
        for index in catalog[s].iter() {
            let cv: &[f64; 3] = catalog[*index].vector.as_ref();
            let c = v.dot(&(*cv).into());
            if let Some((cc, _)) = closest {
                if c > cc {
                    closest = Some((c, *index));
                }
            } else {
                closest = Some((c, *index));
            }
        }
    }
    closest
}

//a StarCalibrateDesc
//tp StarCalibrateDesc
/// A description of a calibration for a camera and a lens, for an
/// image of a grid (e.g. graph paper)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarCalibrateDesc {
    /// Camera description (body, lens, min focus distance)
    camera: CameraPolynomialDesc,
    /// Sensor coordinate, star 'brightness', Hipparcos catalog id
    mappings: Vec<(isize, isize, usize, usize)>,
}

//a StarCalibrate
//tp StarCalibrate
#[derive(Debug, Clone, Default, Serialize)]
pub struct StarCalibrate {
    /// Description of the camera body
    #[serde(serialize_with = "serialize_body_name")]
    body: CameraBody,
    /// The spherical lens mapping polynomial
    #[serde(serialize_with = "serialize_lens_name")]
    lens: CameraLens,
    /// Distance of the focus of the camera in the image
    mm_focus_distance: f64,
    /// Mappings from grid coordinates to absolute camera pixel values
    mappings: Vec<(isize, isize, usize, usize)>,
    /// Derived camera instance
    #[serde(skip)]
    camera: CameraPolynomial,
}

//ip StarCalibrate
impl StarCalibrate {
    //ap camera
    pub fn camera(&self) -> &CameraPolynomial {
        &self.camera
    }

    //ap mappings
    pub fn mappings(&self) -> &[(isize, isize, usize, usize)] {
        &self.mappings
    }

    //cp from_desc
    pub fn from_desc(cdb: &CameraDatabase, desc: StarCalibrateDesc) -> Result<Self> {
        let position = Point3D::default();
        let direction = Quat::default();

        let body = cdb.get_body_err(&desc.camera.body)?.clone();
        let lens = cdb.get_lens_err(&desc.camera.lens)?.clone();
        let camera = CameraPolynomial::new(
            body.clone(),
            lens.clone(),
            desc.camera.mm_focus_distance,
            position,
            direction,
        );
        // eprintln!("{camera}");
        // let m: Point3D = camera.camera_xyz_to_world_xyz([0., 0., -desc.distance].into());
        // eprintln!("Camera {camera} focused on {m}");
        let s = Self {
            body,
            lens,
            camera,
            mm_focus_distance: desc.camera.mm_focus_distance,
            mappings: desc.mappings,
        };
        Ok(s)
    }

    //cp from_json
    pub fn from_json(cdb: &CameraDatabase, json: &str) -> Result<Self> {
        let desc: StarCalibrateDesc = json::from_json("camera calibration descriptor", json)?;
        Self::from_desc(cdb, desc)
    }
}

//a Main
// 16:41:51:2331:~/Git/star-catalog-rs:$ ./target/release/star-catalog hipp_bright image --fov 25 -W 5184 -H 3456 -o a.png -a 300 -r 196.1 -d 53.9
pub fn main() -> Result<()> {
    let command = clap::Command::new("ic_play")
        .about("Camera calibration tool")
        .version("0.1.0");
    let mut build = CommandBuilder::new(command, None);
    build.add_arg(
        clap::Arg::new("camera_db")
            .long("db")
            .alias("database")
            .required(true)
            .help("Camera database JSON")
            .action(clap::ArgAction::Set),
        Box::new(arg_camera_db),
    );
    let mut cmd_args = CmdArgs::default();
    let mut command: CommandSet<CmdArgs> = build.into();
    command.execute_env(&mut cmd_args)?;

    let camera_db_filename = "nac/camera_db.json";
    let read_filename: Option<&str> = None;
    let camera_filename = "nac/camera_calibrate_stars_4924.json";
    let read_filename = Some("/Users/gjstark/Git/Images/IMG_4924.JPG");
    let camera_filename = "nac/camera_calibrate_stars_5005.json";
    let read_filename = Some("/Users/gjstark/Git/Images/IMG_5005.JPG");
    let camera_filename = "nac/camera_calibrate_stars_5006.json";
    let read_filename = Some("/Users/gjstark/Git/Images/IMG_5006.JPG");
    let write_filename: Option<&str> = None;
    let write_filename = Some("a.png");

    let camera_db_json = json::read_file(camera_db_filename)?;
    let mut cdb: CameraDatabase = json::from_json("camera database", &camera_db_json)?;
    cdb.derive();

    let camera_json = json::read_file(camera_filename)?;
    let calibrate = StarCalibrate::from_json(&cdb, &camera_json)?;

    let mut cam = calibrate.camera().clone();
    cam.set_position([0., 0., 0.].into());
    cam.set_orientation(Quat::default());

    //cb Create Vec of unit direction vectors
    /// Apply lens mapping
    let mut star_directions = vec![];
    for (px, py, _mag, _hipp) in calibrate.mappings() {
        let txty = cam.px_abs_xy_to_camera_txty([*px as f64, *py as f64].into());
        star_directions.push(-txty.to_unit_vector());
    }

    //cb Load Hipparocs catalog
    let mut catalog_full: Catalog = postcard::from_bytes(star_catalog::hipparcos::HIPP_BRIGHT_PST)
        .map_err(|e| format!("{e:?}"))?;
    catalog_full.retain(|s, _n| s.brighter_than(6.5));
    catalog_full.sort();
    catalog_full.derive_data();

    //cb Create Vec of stars
    /// Apply lens mapping
    let mut cat_index = vec![];
    for (_px, _py, _mag, hipp) in calibrate.mappings() {
        cat_index.push(catalog_full.find_sorted(*hipp));
    }

    //cb Find candidates for the three stars
    let mut qs = vec![];
    for (i, ci) in cat_index.iter().enumerate() {
        if ci.is_none() {
            continue;
        }
        for (j, cj) in cat_index.iter().enumerate() {
            if i == j {
                continue;
            }
            if cj.is_none() {
                continue;
            }
            let di_m = catalog_full[ci.unwrap()].vector.as_ref();
            let dj_m = catalog_full[cj.unwrap()].vector.as_ref();
            let di_c = star_directions[i];
            let dj_c = star_directions[j];
            qs.push((1.0, orientation_mapping(di_m, dj_m, di_c, dj_c).into()));
        }
    }
    let q_r: Quat = quat::weighted_average_many(qs.iter().copied()).into();
    let q_r_c = q_r.conjugate(); // Get camera-to-model
                                 // Map 0,0,1 camera to model - then we have the direction the camera is looking at
                                 //
                                 // Then right-ascension is atan(y/x), and declination is asin(z)
    let z_axis: Point3D = [0.0, 0., -1.].into(); // z of [1][0] and [2][0] q_r.conjugate()
    let pts_at: Point3D = quat::apply3(q_r_c.as_ref(), z_axis.as_ref()).into();
    eprintln!("Quat {q_r_c}, {pts_at}");
    for i in 0..3 {
        for j in 0..3 {
            eprintln!("{}", pts_at[i].atan2(pts_at[j]).to_degrees());
        }
    }

    let ra = pts_at[1].atan2(pts_at[0]).to_degrees();
    let de = pts_at[2].asin().to_degrees();
    eprintln!("{ra} {de}");

    let mut pts = vec![];
    let s = Subcube::of_vector(&catalog_full[cat_index[0].unwrap()].vector);
    for s in s.iter_range(5) {
        let color = [125, 125, 125, 255].into();
        for index in catalog_full[s].iter() {
            let pt: &[f64; 3] = catalog_full[*index].vector.as_ref();
            pts.push((30.0, (*pt).into(), color));
        }
    }
    for c in &cat_index {
        if c.is_none() {
            continue;
        }
        let color = [0, 255, 255, 255].into();
        let pt: &[f64; 3] = catalog_full[c.unwrap()].vector.as_ref();
        pts.push((20.0, (*pt).into(), color));
    }
    cam.set_orientation(q_r);
    eprintln!("{cam}");
    eprintln!("{cam:?}");

    let mut total_err = 0.;
    for (i, s) in star_directions.iter().enumerate() {
        let q_c_to_m = q_r.conjugate();
        let star_m = quat::apply3(q_c_to_m.as_ref(), s.as_ref());
        if let Some((err, id)) = closest_star(&catalog_full, star_m.into()) {
            if false {
                eprintln!(
                    "{i} {} {} {} {err}",
                    calibrate.mappings()[i].3,
                    catalog_full[id].id,
                    catalog_full[id].mag
                );
            }
            total_err += (1.0 - err).powi(2);
        }
    }
    eprintln!("Total error {:0.4e}", total_err.sqrt());

    if let Some(read_filename) = read_filename {
        let mut img = ImageRgb8::read_image(read_filename)?;
        if let Some(write_filename) = write_filename {
            for (w, p, c) in &pts {
                let mapped = cam.map_model(*p);
                img.draw_cross(mapped, *w, c);
            }
            let color = [255, 0, 255, 255].into();
            for (px, py, _mag, _hipp) in calibrate.mappings() {
                img.draw_cross([*px as f64, *py as f64].into(), 10.0, &color);
            }
            img.write(write_filename)?;
        }
    }

    Ok(())
}
