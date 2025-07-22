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
use ic_base::{Point2D, Point3D, RollYaw, TanXTanY};
use ic_camera::polynomial;
use ic_camera::polynomial::CalcPoly;
use ic_camera::{serialize_body_name, serialize_lens_name};
use ic_camera::{CameraBody, CameraLens, CameraPolynomial, CameraPolynomialDesc};
use ic_camera::{CameraDatabase, CameraInstance, CameraProjection};
use ic_image::{Color, Image, ImageRgb8};

use ic_mapping::{ModelLineSet, NamedPoint, NamedPointSet, PointMappingSet};

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
    pub fn from_desc(cdb: &CameraDatabase, desc: StarCalibrateDesc) -> Result<Self, String> {
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
    pub fn from_json(cdb: &CameraDatabase, json: &str) -> Result<Self, String> {
        let desc: StarCalibrateDesc = json::from_json("camera calibration descriptor", json)?;
        Self::from_desc(cdb, desc)
    }
}

//a Main
// 16:41:51:2331:~/Git/star-catalog-rs:$ ./target/release/star-catalog hipp_bright image --fov 25 -W 5184 -H 3456 -o a.png -a 300 -r 196.1 -d 53.9
pub fn main() -> Result<(), String> {
    let camera_db_filename = "nac/camera_db.json";
    let camera_filename = "nac/camera_calibrate_stars_4924.json";
    let read_filename = Some("/Users/gjstark/Git/Images/IMG_4924.JPG");
    let camera_filename = "nac/camera_calibrate_stars_5006.json";
    let read_filename = Some("/Users/gjstark/Git/Images/IMG_5006.JPG");
    let camera_filename = "nac/camera_calibrate_stars_5005.json";
    let read_filename = Some("/Users/gjstark/Git/Images/IMG_5005.JPG");
    let write_filename = Some("a.png");
    let read_filename: Option<&str> = None;
    let write_filename: Option<&str> = None;

    let camera_db_json = json::read_file(camera_db_filename)?;
    let mut cdb: CameraDatabase = json::from_json("camera database", &camera_db_json)?;
    cdb.derive();

    let camera_json = json::read_file(camera_filename)?;
    let calibrate = StarCalibrate::from_json(&cdb, &camera_json)?;

    let mut cam = calibrate.camera().clone();
    cam.set_position([0., 0., 0.].into());
    cam.set_orientation(Quat::default());

    //cb Create Vec of camera unit direction vectors
    /// Apply lens mapping
    let mut star_directions_c = vec![];
    for (px, py, _mag, _hipp) in calibrate.mappings() {
        let txty = cam.px_abs_xy_to_camera_txty([*px as f64, *py as f64].into());
        star_directions_c.push(-txty.to_unit_vector());
    }

    //cb Load Hipparocs catalog
    // let mut grid_dir_of_xy = HashMap::new();
    // fine for the big ones (mag 2.5 or brighter)
    //
    // Some in big dipper are 3.4 though
    //
    // for others, we need down to 5.8 or 6.1

    let mut catalog: Catalog = postcard::from_bytes(star_catalog::hipparcos::HIPP_BRIGHT_PST)
        .map_err(|e| format!("{e:?}"))?;
    catalog.retain(|s, _n| s.brighter_than(5.5));
    catalog.sort();
    catalog.derive_data();

    //cb Load catalog_full
    let mut catalog_full: Catalog = postcard::from_bytes(star_catalog::hipparcos::HIPP_BRIGHT_PST)
        .map_err(|e| format!("{e:?}"))?;
    catalog_full.retain(|s, _n| s.brighter_than(6.5));
    catalog_full.sort();
    catalog_full.derive_data();

    //cb Create list of mag1_stars and directions to them, and mag2 if possible
    let mut mag1_stars = vec![];
    let mut mag2_stars = vec![];
    for (n, (_px, _py, mag, _hipp)) in calibrate.mappings().iter().enumerate() {
        if *mag == 1 {
            mag1_stars.push(n);
        }
        if *mag == 2 {
            mag2_stars.push(n);
        }
    }
    assert!(mag1_stars.len() >= 3);
    assert!(mag2_stars.len() >= 3);
    let mag1_directions_c: Vec<Point3D> =
        mag1_stars.iter().map(|n| star_directions_c[*n]).collect();
    let mag2_directions_c: Vec<Point3D> =
        mag2_stars.iter().map(|n| star_directions_c[*n]).collect();

    //cb Create angles between first three mag1 stars
    let mag1_angles = [
        mag1_directions_c[0].dot(&mag1_directions_c[1]).acos(),
        mag1_directions_c[1].dot(&mag1_directions_c[2]).acos(),
        mag1_directions_c[2].dot(&mag1_directions_c[0]).acos(),
    ];
    let angle_degrees: Vec<_> = mag1_angles.iter().map(|a| a.to_degrees()).collect();
    eprintln!("angles: {:?}", angle_degrees);

    //cb Find candidates for the three stars
    let subcube_iter = Subcube::iter_all();
    let candidate_tris = catalog.find_star_triangles(subcube_iter, &mag1_angles, 0.003);
    // let candidate_tris = catalog.find_star_triangles(subcube_iter, &mag1_angles, 0.03);
    let mut printed = 0;
    let mut candidate_q_m_to_c = vec![];
    for (n, tri) in candidate_tris.iter().enumerate() {
        let q_m_to_c = orientation_mapping_triangle(
            catalog[tri.0].vector.as_ref(),
            catalog[tri.1].vector.as_ref(),
            catalog[tri.2].vector.as_ref(),
            mag1_directions_c[1],
            mag1_directions_c[0],
            mag1_directions_c[2],
        );

        candidate_q_m_to_c.push((n, q_m_to_c));
        printed += 1;
        if printed < 10 {
            eprintln!(
                "{n}: {}, {}, {}",
                catalog[tri.1].id, catalog[tri.0].id, catalog[tri.2].id,
            );
        }
    }

    //cb Find candidates for the first three mag2 stars if given
    let mut mag2_candidate_q_m_to_c = vec![];
    let mag2_angles = [
        mag2_directions_c[0].dot(&mag2_directions_c[1]).acos(),
        mag2_directions_c[1].dot(&mag2_directions_c[2]).acos(),
        mag2_directions_c[2].dot(&mag2_directions_c[0]).acos(),
    ];
    let angle_degrees: Vec<_> = mag2_angles.iter().map(|a| a.to_degrees()).collect();
    eprintln!("mag2 angles: {:?}", angle_degrees);

    //cb Find candidates for the three stars
    let subcube_iter = Subcube::iter_all();
    let mag2_candidate_tris = catalog.find_star_triangles(subcube_iter, &mag2_angles, 0.003);
    for (n, tri) in mag2_candidate_tris.iter().enumerate() {
        let q_m_to_c = orientation_mapping_triangle(
            catalog[tri.0].vector.as_ref(),
            catalog[tri.1].vector.as_ref(),
            catalog[tri.2].vector.as_ref(),
            mag2_directions_c[1],
            mag2_directions_c[0],
            mag2_directions_c[2],
        );

        mag2_candidate_q_m_to_c.push((n, q_m_to_c));
    }

    //cb Find mag1 that match mag2
    let mut mag1_mag2_pairs = vec![];
    for (n1, mag1_q_m_to_c) in candidate_q_m_to_c.iter() {
        for (n2, mag2_q_m_to_c) in mag2_candidate_q_m_to_c.iter() {
            let q = *mag2_q_m_to_c / *mag1_q_m_to_c;
            let r = q.as_rijk().0;
            if r > 0.9995 {
                let qs = [
                    (1.0, mag1_q_m_to_c.into_array()),
                    (1.0, mag2_q_m_to_c.into_array()),
                ];
                let q_r: Quat = quat::weighted_average_many(qs.into_iter()).into();
                let mag1_tri = &candidate_tris[*n1];
                let mag2_tri = &mag2_candidate_tris[*n2];
                mag1_mag2_pairs.push((r, q_r, *mag1_tri, *mag2_tri));
                eprintln!(
                    "{},{},{} {},{},{} {r} : {}",
                    catalog[mag1_tri.1].id,
                    catalog[mag1_tri.0].id,
                    catalog[mag1_tri.2].id,
                    catalog[mag2_tri.1].id,
                    catalog[mag2_tri.0].id,
                    catalog[mag2_tri.2].id,
                    catalog[mag2_tri.2].de.to_degrees(),
                );
            }
        }
    }
    mag1_mag2_pairs.sort_by(|a, b| (b.0).partial_cmp(&a.0).unwrap());

    //cb Generate results
    let mut printed = 0;
    for (_r, q_r, tri_mag1, tri_mag2) in mag1_mag2_pairs.iter() {
        // for (n, q_m_to_c) in candidate_q_m_to_c.iter() {
        let verbose = printed < 3;
        printed += 1;
        let mut total_error = 0.;
        let q_c_to_m = q_r.conjugate();
        for (i, s) in star_directions_c.iter().enumerate() {
            let star_m = quat::apply3(q_c_to_m.as_ref(), s.as_ref());
            if let Some((err, id)) = closest_star(&catalog_full, star_m.into()) {
                total_error += (1.0 - err).powi(2);
                if verbose {
                    eprintln!(
                        "[{}, {}, {}, {}], // {i} : {err:0.4e} :{}",
                        calibrate.mappings()[i].0,
                        calibrate.mappings()[i].1,
                        calibrate.mappings()[i].2,
                        catalog_full[id].id,
                        catalog_full[id].mag
                    );
                }
            } else if verbose {
                eprintln!("{i} : None");
            }
        }
        if verbose {
            eprintln!(
                "{:0.4e} : {}, {}, {}, {}, {}, {}\n,",
                total_error.sqrt(),
                catalog[tri_mag1.1].id,
                catalog[tri_mag1.0].id,
                catalog[tri_mag1.2].id,
                catalog[tri_mag2.1].id,
                catalog[tri_mag2.0].id,
                catalog[tri_mag2.2].id,
            );
        }
    }

    /*
        if let Some(read_filename) = read_filename {
            let mut img = ImageRgb8::read_image(read_filename)?;
            if let Some(write_filename) = write_filename {
                let c = &[255, 0, 0, 0].into();
                for (_g, p) in &xy_pairs {
                    img.draw_cross(*p, 5.0, c);
                }
                for (p, c) in &pts {
                    let mapped = camera.map_model(*p);
                    if c.0[0] == 100 {
                        let xyz = camera.world_xyz_to_camera_xyz(*p);
                        let txy = camera.world_xyz_to_camera_txty(*p);
                        eprintln!("{mapped} {xyz} {txy} {p} {c:?}");
                    }
                    img.draw_cross(mapped, 5.0, c);
                }
                img.write(write_filename)?;
            }
    }
        */

    Ok(())
}
