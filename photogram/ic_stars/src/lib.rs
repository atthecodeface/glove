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
//!     noticeable offset from each other; repeat from step 3
//!
//!  5. Once the graphs are all deemed reasonable, copy the
//!     polynomials calculated in to the lens mapping.
//!
//!  6. Rerun, and the graphs should be near identity, and the
//!     calibration is complete.
//!  

//a Imports
use geo_nd::{quat, Quaternion, Vector};
use serde::{Deserialize, Serialize};
use star_catalog::{Catalog, CatalogIndex, Subcube};

use ic_base::json;
use ic_base::{Point2D, Point3D, Quat, Result, TanXTanY};
use ic_camera::{serialize_body_name, serialize_lens_name};
use ic_camera::{CameraBody, CameraLens, CameraPolynomial, CameraPolynomialDesc};
use ic_camera::{CameraDatabase, CameraProjection};
use ic_image::{Image, ImageRgb8};

//a ImgPt
//tp ImgPt
pub struct ImgPt {
    px: f32,
    py: f32,
    style: u8,
}
impl std::convert::From<(usize, usize, u8)> for ImgPt {
    fn from((px, py, style): (usize, usize, u8)) -> ImgPt {
        ImgPt {
            px: px as f32,
            py: py as f32,
            style,
        }
    }
}
impl std::convert::From<(isize, isize, u8)> for ImgPt {
    fn from((px, py, style): (isize, isize, u8)) -> ImgPt {
        ImgPt {
            px: px as f32,
            py: py as f32,
            style,
        }
    }
}
impl std::convert::From<(Point2D, u8)> for ImgPt {
    fn from((pt, style): (Point2D, u8)) -> ImgPt {
        ImgPt {
            px: pt[0] as f32,
            py: pt[1] as f32,
            style,
        }
    }
}

impl ImgPt {
    pub fn draw(&self, img: &mut ImageRgb8) {
        let (width, color) = match self.style {
            0 => (10.0, &[255, 0, 255, 255].into()),
            1 => (20.0, &[0, 255, 255, 255].into()),
            _ => (30.0, &[125, 125, 125, 255].into()),
        };
        img.draw_cross([self.px as f64, self.py as f64].into(), width, color);
    }
}

//a Useful functions
//fi orientation_mapping_triangle
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
    let qs = vec![
        (1.0, orientation_mapping(di_m, dj_m, di_c, dj_c).into()),
        (1.0, orientation_mapping(di_m, dk_m, di_c, dk_c).into()),
        (1.0, orientation_mapping(dj_m, dk_m, dj_c, dk_c).into()),
        (1.0, orientation_mapping(dj_m, di_m, dj_c, di_c).into()),
        (1.0, orientation_mapping(dk_m, di_m, dk_c, di_c).into()),
        (1.0, orientation_mapping(dk_m, dj_m, dk_c, dj_c).into()),
    ];
    quat::weighted_average_many(qs.iter().copied()).into()
}

//fp orientation_mapping
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

//fp closest_star
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
/// Star calibration file
///
/// This serializes to a [StarCalibrateDesc] - in theory
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
    /// Directions to all the stars
    star_directions: Vec<Point3D>,
}

//ip StarCalibrate
impl StarCalibrate {
    //ap camera
    pub fn camera(&self) -> &CameraPolynomial {
        &self.camera
    }

    //ap camera_mut
    pub fn camera_mut(&mut self) -> &mut CameraPolynomial {
        &mut self.camera
    }

    //ap mappings
    pub fn mappings(&self) -> &[(isize, isize, usize, usize)] {
        &self.mappings
    }

    //ap star_directions
    pub fn star_directions(&self) -> &[Point3D] {
        &self.star_directions
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
        let s = Self {
            body,
            lens,
            camera,
            mm_focus_distance: desc.camera.mm_focus_distance,
            mappings: desc.mappings,
            star_directions: vec![],
        };
        Ok(s)
    }

    //cp from_json
    pub fn from_json(cdb: &CameraDatabase, json: &str) -> Result<Self> {
        let desc: StarCalibrateDesc = json::from_json("camera calibration descriptor", json)?;
        Self::from_desc(cdb, desc)
    }

    //mp recalculate_star_directions
    /// Create Vec of camera unit direction vectors
    ///
    /// Maps the absolute pixel px,py to camera direction, ignoring the orientation of the camera
    ///
    /// It *does* apply the lens mapping of the camera
    pub fn recalculate_star_directions(&mut self) {
        self.star_directions = vec![];
        for (px, py, _mag, _hipp) in &self.mappings {
            let txty = self
                .camera
                .px_abs_xy_to_camera_txty([*px as f64, *py as f64].into());
            self.star_directions.push(-txty.to_unit_vector());
        }
    }

    //mp find_stars_in_catalog
    /// Maps each entry to a CatalogIndex (or none if not found)
    pub fn find_stars_in_catalog(&self, catalog: &Catalog) -> Vec<Option<CatalogIndex>> {
        self.mappings
            .iter()
            .map(|(_, _, _, id)| catalog.find_sorted(*id))
            .collect()
    }

    //mp set_orientation
    pub fn set_orientation(&mut self, q: Quat, verbose: bool) {
        self.camera.set_position([0., 0., 0.].into());
        self.camera.set_orientation(q);
        if verbose {
            let q_r_c = q.conjugate(); // Get camera-to-model
                                       // Map 0,0,1 camera to model - then we have the direction the camera is looking at
                                       //
                                       // Then right-ascension is atan(y/x), and declination is asin(z)
            let z_axis: Point3D = [0.0, 0., -1.].into(); // z of [1][0] and [2][0] q_r.conjugate()
            let pts_at: Point3D = quat::apply3(q_r_c.as_ref(), z_axis.as_ref()).into();
            let ra = pts_at[1].atan2(pts_at[0]).to_degrees();
            let de = pts_at[2].asin().to_degrees();
            eprintln!("Camera seems to point at right-ascension {ra:0.3} declination {de:0.3}");
            eprintln!("Camera : {}", self.camera());
            eprintln!(" orientation: {q}");
        }
    }

    //mp show_star_mappings
    /// Return a Vec of the *expected* positions of the stars in the
    /// calibration on the camera sensor
    pub fn show_star_mappings(&self, catalog: &Catalog) -> Vec<Point2D> {
        let mut pts = vec![];

        eprintln!(" [px, py, mag, catalog_id] - suitable for use in calibrate.json file",);

        let mut total_error = 0.;
        for (i, s) in self.star_directions.iter().enumerate() {
            let star_m = self.camera.camera_xyz_to_world_xyz(*s);
            if let Some((err, id)) = closest_star(catalog, star_m) {
                let star = &catalog[id];
                let sv: &[f64; 3] = star.vector.as_ref();
                let star_pxy = self.camera.world_xyz_to_px_abs_xy((*sv).into());
                pts.push(star_pxy);
                total_error += (1.0 - err).powi(2);
                eprintln!(
                    "[{}, {}, {}, {}], // {i} : {err:0.4e} : mag {} : [{}, {}]",
                    self.mappings[i].0,
                    self.mappings[i].1,
                    self.mappings[i].2,
                    star.id,
                    star.mag,
                    star_pxy[0] as isize,
                    star_pxy[1] as isize,
                );
            } else {
                eprintln!(
                    "[{}, {}, {}, 0], // {i} : No mapping",
                    self.mappings[i].0, self.mappings[i].1, self.mappings[i].2,
                );
            }
        }
        eprintln!("\nTotal error {:0.4e}", total_error.sqrt(),);
        pts
    }

    //mp Map stars in catalog, and plot them on an image
    pub fn map_stars(
        &mut self,
        catalog: &mut Catalog,
        search_brightness: f32,
    ) -> Result<Vec<Option<CatalogIndex>>> {
        catalog.retain(move |s, _n| s.brighter_than(search_brightness));
        catalog.sort();
        catalog.derive_data();

        self.recalculate_star_directions();
        self.camera.set_position([0., 0., 0.].into());

        let cat_index = self.find_stars_in_catalog(catalog);

        //cb Find orientations for every pair of *mapped* stars
        let mut qs = vec![];
        for (i, ci) in cat_index.iter().enumerate() {
            if ci.is_none() {
                continue;
            }
            let di_m = catalog[ci.unwrap()].vector.as_ref();
            let di_c = self.star_directions[i];
            for (j, cj) in cat_index.iter().enumerate() {
                if i == j {
                    continue;
                }
                if cj.is_none() {
                    continue;
                }
                let dj_m = catalog[cj.unwrap()].vector.as_ref();
                let dj_c = self.star_directions[j];
                qs.push((1.0, orientation_mapping(di_m, dj_m, di_c, dj_c).into()));
            }
        }

        //cb Get best orientation (mapping from model-to-camera), and the reverse
        let q_r: Quat = quat::weighted_average_many(qs.iter().copied()).into();
        self.set_orientation(q_r, true);

        Ok(cat_index)
    }

    //mp find_stars_from_image
    // 16:41:51:2331:~/Git/star-catalog-rs:$ ./target/release/star-catalog hipp_bright image --fov 25 -W 5184 -H 3456 -o a.png -a 300 -r 196.1 -d 53.9
    pub fn find_stars_from_image(
        &mut self,
        catalog: &mut Catalog,
        search_brightness: f32,
    ) -> Result<()> {
        catalog.retain(move |s, _n| s.brighter_than(search_brightness));
        catalog.sort();
        catalog.derive_data();

        self.recalculate_star_directions();

        //cb Load catalog_full - should use max_brightness
        let mut catalog_full: Catalog =
            postcard::from_bytes(star_catalog::hipparcos::HIPP_BRIGHT_PST)
                .map_err(|e| format!("{e:?}"))?;
        catalog_full.retain(|s, _n| s.brighter_than(6.5));
        catalog_full.sort();
        catalog_full.derive_data();

        //cb Create list of mag1_stars and directions to them, and mag2 if possible
        let mut mag1_stars = vec![];
        let mut mag2_stars = vec![];
        for (n, (_px, _py, mag, _hipp)) in self.mappings.iter().enumerate() {
            if *mag == 1 {
                mag1_stars.push(n);
            }
            if *mag == 2 {
                mag2_stars.push(n);
            }
        }

        if mag1_stars.len() < 3 || mag2_stars.len() < 3 {
            return Err(format!(
            "The calibration requires three 'mag 1' and three 'mag 2' stars; there were {} and {}",
            mag1_stars.len(),
            mag2_stars.len()
        )
            .into());
        }

        let mag1_directions_c: Vec<Point3D> = mag1_stars
            .iter()
            .map(|n| self.star_directions[*n])
            .collect();
        let mag2_directions_c: Vec<Point3D> = mag2_stars
            .iter()
            .map(|n| self.star_directions[*n])
            .collect();

        //cb Create angles between first three mag1 stars
        let mag1_angles = [
            mag1_directions_c[0].dot(&mag1_directions_c[1]).acos(),
            mag1_directions_c[1].dot(&mag1_directions_c[2]).acos(),
            mag1_directions_c[2].dot(&mag1_directions_c[0]).acos(),
        ];
        let angle_degrees: Vec<_> = mag1_angles.iter().map(|a| a.to_degrees()).collect();
        eprintln!(
        "Angles (just using focal length of lens) between first three magnitude '1' stars: {angle_degrees:?}"
        
        );

        //cb Create angles between first three mag2 stars
        let mag2_angles = [
            mag2_directions_c[0].dot(&mag2_directions_c[1]).acos(),
            mag2_directions_c[1].dot(&mag2_directions_c[2]).acos(),
            mag2_directions_c[2].dot(&mag2_directions_c[0]).acos(),
        ];
        let angle_degrees: Vec<_> = mag2_angles.iter().map(|a| a.to_degrees()).collect();
        eprintln!(
        "Angles (just using focal length of lens) between first three magnitude '2' stars: {angle_degrees:?}"
        );

        //cb Find candidates for the three stars
        let subcube_iter = Subcube::iter_all();
        let candidate_tris = catalog.find_star_triangles(subcube_iter, &mag1_angles, 0.003);
        // let candidate_tris = catalog.find_star_triangles(subcube_iter, &mag1_angles, 0.03);
        let mut printed = 0;
        let mut candidate_q_m_to_c = vec![];
        eprintln!(
            "\nGenerating candidate StarCatalog 'id's for the three stars for magnitude 1 triangle",
        );
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
            match printed.cmp(&10) {
                std::cmp::Ordering::Equal => {
                    eprintln!("...");
                }
                std::cmp::Ordering::Less => {
                    eprintln!(
                        "{n}: {}, {}, {}",
                        catalog[tri.1].id, catalog[tri.0].id, catalog[tri.2].id,
                    );
                }
                _ => {}
            }
        }
        eprintln!("Total: {} candidates", candidate_q_m_to_c.len());

        //cb Find candidates for the first three mag2 stars if given
        let mut mag2_candidate_q_m_to_c = vec![];

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
        eprintln!("\nFinding matching orientations for magnitude 1 and magnitude 2 candidates",);
        let mut printed = 0;
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
                    printed += 1;
                    match printed.cmp(&10) {
                        std::cmp::Ordering::Equal => {
                            eprintln!("...");
                        }
                        std::cmp::Ordering::Less => {
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
                        _ => {}
                    }
                }
            }
        }
        eprintln!("Total: {} matching orientations", mag1_mag2_pairs.len());
        mag1_mag2_pairs.sort_by(|a, b| (b.0).partial_cmp(&a.0).unwrap());

        //cb Generate results
        let (_r, q_r, tri_mag1, tri_mag2) = mag1_mag2_pairs[0];
        self.set_orientation(q_r, true);

        eprintln!("\nThe best match of the candidate triangles:");
        eprintln!(
            "    {}, {}, {}, {}, {}, {},",
            catalog[tri_mag1.1].id,
            catalog[tri_mag1.0].id,
            catalog[tri_mag1.2].id,
            catalog[tri_mag2.1].id,
            catalog[tri_mag2.0].id,
            catalog[tri_mag2.2].id,
        );
        Ok(())
    }

    //mp add_catalog_stars
    pub fn add_catalog_stars(&self, catalog: &Catalog, mapped_pts: &mut Vec<ImgPt>) -> Result<()> {
        //cb Create Vec<Model> of all the catalog stars (that are in-front of the camera)
        for s in Subcube::iter_all() {
            for index in catalog[s].iter() {
                let pt: &[f64; 3] = catalog[*index].vector.as_ref();
                let mapped = self.camera.world_xyz_to_camera_xyz((*pt).into());
                if mapped[2] < -0.05 {
                    let camera_txty: TanXTanY = mapped.into();
                    mapped_pts.push((self.camera.camera_txty_to_px_abs_xy(&camera_txty), 2).into());
                }
            }
        }
        Ok(())
    }
    pub fn add_cat_index(
        &self,
        catalog: &Catalog,
        cat_index: &[Option<CatalogIndex>],
        mapped_pts: &mut Vec<ImgPt>,
    ) -> Result<()> {
        //cb Add (in pink) the Calibration stars that map to a Catalog star
        for c in cat_index {
            if c.is_none() {
                continue;
            }
            let pt: &[f64; 3] = catalog[c.unwrap()].vector.as_ref();
            let mapped = self.camera.world_xyz_to_px_abs_xy((*pt).into());
            mapped_pts.push((mapped, 1).into());
        }
        Ok(())
    }

    //mp add_mapping_pts
    /// Add point for each 'mapping' in this calibration, to mapped_pts vector
    pub fn add_mapping_pts(&self, mapped_pts: &mut Vec<ImgPt>) -> Result<()> {
        //cb Create mapped points
        for (px, py, _mag, _hipp) in self.mappings() {
            mapped_pts.push((*px, *py, 0).into());
        }
        Ok(())
    }
}
