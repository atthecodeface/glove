//a Imports
use geo_nd::{quat, Quaternion, Vector};
use serde::{Deserialize, Serialize};
use star_catalog::{Catalog, CatalogIndex, Subcube};

use ic_base::json;
use ic_base::{Point2D, Point3D, Quat, Result, RollYaw, TanXTanY};
use ic_camera::CameraProjection;
use ic_camera::{CalibrationMapping, CameraInstance};
use ic_image::ImagePt;

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
) -> (Quat, f64) {
    let qs = vec![
        (1.0, orientation_mapping(di_m, dj_m, di_c, dj_c).into()),
        (1.0, orientation_mapping(di_m, dk_m, di_c, dk_c).into()),
        (1.0, orientation_mapping(dj_m, dk_m, dj_c, dk_c).into()),
        (1.0, orientation_mapping(dj_m, di_m, dj_c, di_c).into()),
        (1.0, orientation_mapping(dk_m, di_m, dk_c, di_c).into()),
        (1.0, orientation_mapping(dk_m, dj_m, dk_c, dj_c).into()),
    ];
    let q_avg: Quat = quat::weighted_average_many(qs.iter().copied()).into();
    let mut err = 0.0;
    let q_c = q_avg.conjugate();
    for q in qs {
        let q: Quat = q.1.into();
        let q = q * q_c;
        let r = q.as_rijk().0.abs();
        err += 1.0 - r;
    }
    (q_avg, err)
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
    let s = Subcube::of_vector(&v.into_array());
    let mut closest = None;
    for s in s.iter_range(2) {
        for index in catalog[s].iter() {
            let cv: &[f64; 3] = catalog[*index].vector();
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

//a StarMapping
//tp StarMapping
/// Should probably store this as a vec of Point3D and a vec of same length of Point2D
#[derive(Debug, Clone, Default)]
pub struct StarMapping {
    /// Sensor coordinate, star 'brightness', Hipparcos catalog id
    mappings: Vec<(isize, isize, usize, usize)>,
}

//ip Serialize for StarMapping
impl Serialize for StarMapping {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.mappings.len()))?;
        for m in &self.mappings {
            seq.serialize_element(m)?;
        }
        seq.end()
    }
}

//ip Deserialize for StarMapping
impl<'de> Deserialize<'de> for StarMapping {
    fn deserialize<DE>(deserializer: DE) -> std::result::Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let mappings = Vec::<_>::deserialize(deserializer)?;
        Ok(Self { mappings })
    }
}

//ip StarMapping - Constructors and Destructors
impl StarMapping {
    //cp from_json
    pub fn from_json(json: &str) -> Result<Self> {
        json::from_json("star mapping", json)
    }

    //dp to_json
    pub fn to_json(self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self)?)
    }
}

//ip StarMapping - Accessors
impl StarMapping {
    //ap mappings
    pub fn mappings(&self) -> &[(isize, isize, usize, usize)] {
        &self.mappings
    }

    //mp star_direction
    /// Maps the absolute pixel px,py to world direction
    pub fn star_direction(&self, camera: &CameraInstance, index: usize) -> Point3D {
        let (px, py, _, _) = self.mappings[index];
        let txty = camera.px_abs_xy_to_camera_txty([px as f64, py as f64].into());
        camera.camera_xyz_to_world_dir(-txty.to_unit_vector()) // possibly -ve
    }

    //mp mapped_camera_direction
    /// Maps the absolute pixel px,py to camera direction
    ///
    /// It *does* apply the lens mapping of the camera
    pub fn mapped_camera_direction(&self, camera: &CameraInstance, index: usize) -> Point3D {
        let (px, py, _, _) = self.mappings[index];
        let txty = camera.px_abs_xy_to_camera_txty([px as f64, py as f64].into());
        -txty.to_unit_vector()
    }
}

//ip StarMapping
impl StarMapping {
    //mp find_stars_in_catalog
    /// Maps each entry to a CatalogIndex (or none if not found)
    pub fn find_stars_in_catalog(&self, catalog: &Catalog) -> Vec<Option<CatalogIndex>> {
        self.mappings
            .iter()
            .map(|(_, _, _, id)| catalog.find_sorted(*id))
            .collect()
    }

    //mp camera_ra_de
    pub fn camera_ra_de(camera: &CameraInstance) -> (f64, f64) {
        let q_r_c = camera.orientation().conjugate(); // Get camera-to-model
                                                      // Map 0,0,1 camera to model - then we have the direction the camera is looking at
                                                      //
                                                      // Then right-ascension is atan(y/x), and declination is asin(z)
        let z_axis: Point3D = [0.0, 0., -1.].into(); // z of [1][0] and [2][0] q_r.conjugate()
        let pts_at: Point3D = quat::apply3(q_r_c.as_ref(), z_axis.as_ref()).into();
        let ra = pts_at[1].atan2(pts_at[0]).to_degrees();
        let de = pts_at[2].asin().to_degrees();
        (ra, de)
        // eprintln!("Camera seems to point at right-ascension {ra:0.3} declination {de:0.3}");
    }

    //mp update_star_mappings
    /// Update the stars that the mappings map to, within the catalog
    ///
    /// The catalog star has to be within the 'closeness' of its
    /// mapping, and only mappings within 'within' angles of the
    /// orientation are allowed to be mapped.
    ///
    /// All stars that are *not* mapped here are updated to be unmapped
    pub fn update_star_mappings(
        &mut self,
        catalog: &Catalog,
        camera: &CameraInstance,
        close_enough_roll: f64,
        yaw_max_rel_error: f64,
        yaw_min: f64,
        yaw_max: f64,
    ) -> (usize, f64) {
        let yaw_min = yaw_min.to_radians();
        let yaw_max = yaw_max.to_radians();
        let mut num_unmapped = 0;
        let mut total_error = 0.;
        for i in 0..self.mappings.len() {
            let (px, py, _, _) = self.mappings[i];
            let cam_txty = camera.px_abs_xy_to_camera_txty([px as f64, py as f64].into());
            let cam_ry: RollYaw = cam_txty.into();
            let star_m = self.star_direction(camera, i);
            let mut okay = false;
            if cam_ry.yaw() < yaw_min || cam_ry.yaw() > yaw_max {
            } else {
                let subcube_iter = Subcube::iter_all();
                if let Some((err, id)) = catalog.closest_to_dir(subcube_iter, star_m.as_ref()) {
                    let star = &catalog[id];
                    let sv: Point3D = (*star.vector()).into();
                    let model_pxy = camera.world_xyz_to_px_abs_xy(sv);
                    let model_txty = camera.world_xyz_to_camera_txty(sv);
                    let model_ry: RollYaw = model_txty.into();
                    let mut close_enough = false;
                    let relative_yaw_error = model_ry.yaw() / cam_ry.yaw() - 1.0;
                    if relative_yaw_error.abs() < yaw_max_rel_error
                        && (model_ry.roll() - cam_ry.roll()).abs() < close_enough_roll.to_radians()
                    {
                        close_enough = true;
                    }
                    let dpx = model_pxy[0] - (px as f64);
                    let dpy = model_pxy[1] - (py as f64);
                    let dx2 = dpx * dpx + dpy * dpy;
                    if dx2 < 20.0 {
                        close_enough = true;
                    }
                    if close_enough {
                        self.mappings[i].3 = star.id();
                        total_error += (1.0 - err).powi(2);
                        okay = true;
                    }
                }
            }
            if !okay {
                self.mappings[i].3 = 0;
                num_unmapped += 1;
            }
        }
        (num_unmapped, total_error.sqrt())
    }

    //mp show_star_mappings
    /// Shoe them *given* an orientation
    pub fn show_star_mappings(&self, catalog: &Catalog, camera: &CameraInstance) {
        let mut total_error = 0.;
        let mut num_mapped = 0;
        for (i, mapping) in self.mappings.iter().enumerate() {
            if mapping.3 != 0 {
                let star_m = self.star_direction(camera, i);
                if let Some(c) = catalog.find_sorted(mapping.3) {
                    let star = &catalog[c];
                    let sv: Point3D = (*star.vector()).into();
                    let (px, py, _, _) = self.mappings[i];
                    let cam_txty = camera.px_abs_xy_to_camera_txty([px as f64, py as f64].into());
                    let cam_ry: RollYaw = cam_txty.into();
                    let model_txty = camera.world_xyz_to_camera_txty(sv);
                    let model_ry: RollYaw = model_txty.into();
                    let yaw_error = model_ry.yaw() - cam_ry.yaw();
                    let roll_error = model_ry.roll() - cam_ry.roll();
                    let relative_yaw_error = model_ry.yaw() / cam_ry.yaw() - 1.0;

                    let err = sv.dot(&star_m).acos().to_degrees();
                    let star_pxy = camera.world_xyz_to_px_abs_xy(sv);
                    total_error += (1.0 - err).powi(2);
                    num_mapped += 1;
                    println!(
                        "{i:4} pxy [{}, {}] currently maps to {} mag {} with yaw err {:0.2} rel {:0.4e} roll err {:0.2} expected at [{}, {}]",
                        mapping.0,
                        mapping.1,
                        star.id(),
                        star.magnitude(),
                        yaw_error.to_degrees(),
                        relative_yaw_error,
                        roll_error.to_degrees(),
                        star_pxy[0] as isize,
                        star_pxy[1] as isize,
                    );
                } else {
                    println!(
                        "{i:4} pxy [{}, {}] currently maps to id {} which is not in the caalog",
                        mapping.0, mapping.1, mapping.3
                    );
                }
            } else {
                println!(
                    "{i:4} pxy [{}, {}] is not currently mapped",
                    mapping.0, mapping.1
                );
            }
        }
        println!(
            "\nTotal error of {num_mapped} mapped stars out of {} {:0.4e}, mean error {:0.4e} ",
            self.mappings().len(),
            total_error.sqrt(),
            total_error.sqrt() / (num_mapped as f64),
        );
    }

    //mp get_mapped_stars
    /// Return a Vec of the *expected* positions of the stars in the
    /// calibration on the camera sensor
    pub fn get_mapped_stars(
        &mut self,
        catalog: &Catalog,
        camera: &CameraInstance,
        close_enough: f64,
    ) -> Vec<Point2D> {
        let cos_close_enough = close_enough.to_radians().cos();
        let mut pts = vec![];
        for i in 0..self.mappings.len() {
            let star_m = self.star_direction(camera, i);
            if let Some((err, id)) = closest_star(catalog, star_m) {
                if err < cos_close_enough {
                    continue;
                }
                let star = &catalog[id];
                let sv: &[f64; 3] = star.vector();
                let star_pxy = camera.world_xyz_to_px_abs_xy((*sv).into());
                pts.push(star_pxy);
            }
        }
        pts
    }

    //mp create_calibration_mapping
    pub fn create_calibration_mapping(&self, catalog: &Catalog) -> CalibrationMapping {
        let mut world = vec![];
        let mut sensor = vec![];
        for mapping in &self.mappings {
            if mapping.3 != 0 {
                if let Some(c) = catalog.find_sorted(mapping.3) {
                    let star = &catalog[c];
                    let sv: &[f64; 3] = star.vector();

                    // Note that the *distance* is not important; the
                    // 3D point of the mapping is always converted to
                    // a direction (subtracting the camera position)
                    // and reoriented for the camera, and the apparent
                    // distance is irrelevant
                    let sv = [-sv[0], -sv[1], -sv[2]];
                    let map = [mapping.0 as f64, mapping.1 as f64].into();
                    world.push(sv.into());
                    sensor.push(map);
                }
            }
        }
        CalibrationMapping::new(world, sensor)
    }

    //mp find_orientation_from_all_mapped_stars
    /// Find all the mappings' stars in catalog
    ///
    /// Find the direction to each, ignoring the camera orientation
    ///
    /// Find the optimal orientation that the camera should be at using pairs
    pub fn find_orientation_from_all_mapped_stars(
        &self,
        catalog: &Catalog,
        camera: &CameraInstance,
        search_brightness: f32,
    ) -> Result<Quat> {
        //cb Find orientations for every pair of *mapped* stars
        let mut qs = vec![];
        let mut cat_index = vec![];
        for (i, mapping) in self.mappings.iter().enumerate() {
            if let Some(c) = catalog.find_sorted(mapping.3) {
                if catalog[c].magnitude() < search_brightness {
                    cat_index.push((i, c));
                }
            }
        }
        if cat_index.len() < 2 {
            return Err("Could not find 2 stars that map".into());
        }
        for (i, ci) in &cat_index {
            let di_m = catalog[*ci].vector();
            let di_c = self.mapped_camera_direction(camera, *i);
            for (j, cj) in &cat_index {
                if *i == *j {
                    continue;
                }
                let dj_m = catalog[*cj].vector();
                let dj_c = self.mapped_camera_direction(camera, *j);
                qs.push((1.0, orientation_mapping(di_m, dj_m, di_c, dj_c).into()));
            }
        }

        //cb Get best orientation (mapping from model-to-camera), and the reverse
        let q_r: Quat = quat::weighted_average_many(qs.iter().copied()).into();
        Ok(q_r)
    }

    //mp find_orientation_from_triangles
    fn find_stars(
        &self,
        catalog: &Catalog,
        camera: &CameraInstance,
        max_angle_delta: f64,
        mag: usize,
    ) -> Result<Vec<(Quat, (CatalogIndex, CatalogIndex, CatalogIndex), f64)>> {
        //cb Create list of mag1_stars and directions to them, and mag2 if possible
        let mut selected_mappings = vec![];
        for i in 0..self.mappings.len() {
            if self.mappings[i].2 == mag {
                selected_mappings.push(i);
            }
        }

        if selected_mappings.len() < 3 {
            return Err(format!(
                "The calibration requires three 'mag {mag}' star; there were {}",
                selected_mappings.len()
            )
            .into());
        }

        let sensor_directions: Vec<Point3D> = selected_mappings
            .iter()
            .map(|n| self.mapped_camera_direction(camera, *n))
            .collect();

        //cb Create angles between first three mag1 stars
        let sensor_angles = [
            sensor_directions[1].dot(&sensor_directions[2]).acos(),
            sensor_directions[2].dot(&sensor_directions[0]).acos(),
            sensor_directions[0].dot(&sensor_directions[1]).acos(),
        ];
        let angle_degrees: Vec<_> = sensor_angles.iter().map(|a| a.to_degrees()).collect();
        eprintln!("Angles (just using focal length of lens) between first three magnitude '{mag}' stars: {angle_degrees:?}" );

        //cb Find candidates for the three stars
        let subcube_iter = Subcube::iter_all();
        let candidate_tris =
            catalog.find_star_triangles(subcube_iter, &sensor_angles, max_angle_delta);

        let mut printed = 0;
        let mut candidate_q_m_to_c = vec![];
        eprintln!(
            "\nGenerating candidate StarCatalog 'id's for the three stars for magnitude {mag} triangle",
        );
        for tri in candidate_tris.iter() {
            let (q_m_to_c, err) = orientation_mapping_triangle(
                catalog[tri.0].vector(),
                catalog[tri.1].vector(),
                catalog[tri.2].vector(),
                sensor_directions[0],
                sensor_directions[1],
                sensor_directions[2],
            );
            if err < 0.01 {
                candidate_q_m_to_c.push((q_m_to_c, *tri, err));
            }
        }
        candidate_q_m_to_c.sort_by(|a, b| (a.2).partial_cmp(&b.2).unwrap());

        for (_q, tri, e) in &candidate_q_m_to_c {
            printed += 1;
            match printed.cmp(&10) {
                std::cmp::Ordering::Equal => {
                    eprintln!("...");
                }
                std::cmp::Ordering::Less => {
                    eprintln!(
                        "{e:.6e}: {}, {}, {}",
                        catalog[tri.0].id(),
                        catalog[tri.1].id(),
                        catalog[tri.2].id(),
                    );
                }
                _ => {}
            }
        }
        eprintln!("Total: {} candidates", candidate_q_m_to_c.len());
        Ok(candidate_q_m_to_c)
    }

    //mp find_orientation_from_triangles
    /// A value of 0.15 degrees is normal for max_angle_delta
    pub fn find_orientation_from_triangles(
        &mut self,
        catalog: &Catalog,
        camera: &CameraInstance,
        max_angle_delta: f64,
    ) -> Result<Vec<(f64, Quat)>> {
        let max_angle_delta = max_angle_delta.to_radians();

        //cb Create list of mag1_stars and directions to them, and mag2 if possible
        let candidate_q_m_to_c = self.find_stars(catalog, camera, max_angle_delta, 1)?;
        let mag2_candidate_q_m_to_c = self.find_stars(catalog, camera, max_angle_delta, 2)?;

        //cb Find mag1 that match mag2
        eprintln!("\nFinding matching orientations for magnitude 1 and magnitude 2 candidates",);
        let mut printed = 0;
        let mut mag1_mag2_pairs = vec![];
        for (mag1_q_m_to_c, mag1_tri, _e1) in candidate_q_m_to_c.iter() {
            for (mag2_q_m_to_c, mag2_tri, _e2) in mag2_candidate_q_m_to_c.iter() {
                let q = *mag2_q_m_to_c / *mag1_q_m_to_c;
                let r = q.as_rijk().0.abs();
                // r = cos(x = angle of rotation) = 1 - x^2/2 + x^4/24 - ...
                // x^2 = 2(1-r)
                let x2 = (1.0 - r) * 2.0;
                if x2 < max_angle_delta * max_angle_delta {
                    // 3.1E-5 {
                    // x = 0.2 degrees
                    let qs = [
                        (1.0, mag1_q_m_to_c.into_array()),
                        (1.0, mag2_q_m_to_c.into_array()),
                    ];
                    let q_r: Quat = quat::weighted_average_many(qs.into_iter()).into();
                    mag1_mag2_pairs.push((x2.sqrt(), q_r, *mag1_tri, *mag2_tri));
                    printed += 1;
                    match printed.cmp(&10) {
                        std::cmp::Ordering::Equal => {
                            eprintln!("...");
                        }
                        std::cmp::Ordering::Less => {
                            eprintln!(
                                "{} : {},{},{} {},{},{} {r} : {}",
                                x2.sqrt().to_degrees(),
                                catalog[mag1_tri.0].id(),
                                catalog[mag1_tri.1].id(),
                                catalog[mag1_tri.2].id(),
                                catalog[mag2_tri.0].id(),
                                catalog[mag2_tri.1].id(),
                                catalog[mag2_tri.2].id(),
                                catalog[mag2_tri.2].de().to_degrees(),
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
        eprintln!("Total: {} matching orientations", mag1_mag2_pairs.len());
        if mag1_mag2_pairs.is_empty() {
            return Err("Failed to find matching candidate triangles".into());
        }
        mag1_mag2_pairs.sort_by(|a, b| (a.0).partial_cmp(&b.0).unwrap());

        //cb Generate results
        let (x, q_r, tri_mag1, tri_mag2) = &mag1_mag2_pairs[0];

        eprintln!("\nThe best match of the candidate triangles:");
        eprintln!(
            "    {}, {}, {}, {}, {}, {}  : {} {q_r},",
            catalog[tri_mag1.1].id(),
            catalog[tri_mag1.0].id(),
            catalog[tri_mag1.2].id(),
            catalog[tri_mag2.1].id(),
            catalog[tri_mag2.0].id(),
            catalog[tri_mag2.2].id(),
            x.to_degrees(),
        );
        Ok(mag1_mag2_pairs
            .into_iter()
            .map(|(x, q_r, _, _)| (x, q_r))
            .collect())
    }

    //mp img_pts_add_catalog_stars
    pub fn img_pts_add_catalog_stars(
        &self,
        catalog: &Catalog,
        camera: &CameraInstance,
        mapped_pts: &mut Vec<ImagePt>,
        within: f64,
        style: u8,
    ) -> Result<()> {
        let within = within.to_radians();
        for s in Subcube::iter_all() {
            for index in catalog[s].iter() {
                if !catalog.is_filtered(&catalog[*index], 0) {
                    continue;
                }
                let pt: &[f64; 3] = catalog[*index].vector();
                let mapped = camera.world_xyz_to_camera_xyz((*pt).into());
                if mapped[2] < -0.02 {
                    let camera_txty: TanXTanY = mapped.into();
                    let ry: RollYaw = camera_txty.into();
                    if ry.yaw() < within {
                        let pxy = camera.world_xyz_to_px_abs_xy((*pt).into());
                        mapped_pts.push((pxy, style).into());
                    }
                }
            }
        }
        Ok(())
    }

    //mp img_pts_add_cat_index
    pub fn img_pts_add_cat_index(
        &self,
        catalog: &Catalog,
        camera: &CameraInstance,
        mapped_pts: &mut Vec<ImagePt>,
        style: u8,
    ) -> Result<()> {
        //cb Add (in blue) the Calibration stars that map to a Catalog star
        for mapping in &self.mappings {
            if let Some(c) = catalog.find_sorted(mapping.3) {
                let pt: &[f64; 3] = catalog[c].vector();
                let mapped = camera.world_xyz_to_px_abs_xy((*pt).into());
                mapped_pts.push((mapped, style).into());
            }
        }
        Ok(())
    }

    //mp img_pts_add_mapping_pxy
    /// Add (in pink) point for each 'mapping' in this calibration, to mapped_pts vector
    pub fn img_pts_add_mapping_pxy(&self, mapped_pts: &mut Vec<ImagePt>, style: u8) -> Result<()> {
        for (px, py, _mag, _hipp) in self.mappings() {
            mapped_pts.push((*px, *py, style).into());
        }
        Ok(())
    }
}
