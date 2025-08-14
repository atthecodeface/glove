//a Imports
use std::collections::HashSet;
use std::rc::Rc;

use geo_nd::Vector;
use serde::{Deserialize, Serialize};

use ic_base::{json, utils, Error, Point2D, Ray, Result};
use ic_camera::CameraProjection;

use crate::{ModelLineSet, NamedPoint, NamedPointSet, PointMapping};

//a PmsGoodScreenPair
#[derive(Default, Debug, Clone)]
struct PmsGoodScreenPairPt {
    pms_index: usize,
    use_count: usize,
    length: f64,
    pxy: Point2D,
}

impl PmsGoodScreenPairPt {
    fn new(pms_index: usize, cog: &Point2D, pms: &PointMapping) -> Self {
        let pxy = pms.screen() - *cog;
        let length = pxy.length();
        Self {
            pms_index,
            use_count: 0,
            length,
            pxy,
        }
    }
}

use std::collections::VecDeque;
#[derive(Default)]
struct PmsGoodScreenPairSet {
    pts: VecDeque<PmsGoodScreenPairPt>,
    used_dpxy: Vec<Point2D>,
    used_pairs: HashSet<(usize, usize)>,
}

impl PmsGoodScreenPairSet {
    fn add_pt(&mut self, pms_index: usize, cog: &Point2D, pms: &PointMapping) {
        self.pts
            .push_back(PmsGoodScreenPairPt::new(pms_index, cog, pms));
    }

    fn sort_pts(&mut self) -> &mut Self {
        self.pts
            .make_contiguous()
            .sort_by(|a, b| b.length.partial_cmp(&a.length).unwrap());
        self
    }
}

impl Iterator for PmsGoodScreenPairSet {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<(usize, usize)> {
        let mut best_index_d = (0., 0, 0, Point2D::default());
        let pms_0 = self.pts[0].pms_index;
        for i in 1..self.pts.len() {
            let pms_i = self.pts[i].pms_index;
            if self.used_pairs.contains(&(pms_0, pms_i)) {
                continue;
            }

            let dpxy = self.pts[0].pxy - self.pts[i].pxy;
            let mut d = dpxy.length();
            for used_dpxy in &self.used_dpxy {
                d += (dpxy[0] * used_dpxy[1] - dpxy[1] * used_dpxy[0]).abs();
            }
            let d = d / ((self.pts[i].use_count + 1) as f64);
            if d > best_index_d.0 {
                best_index_d = (d, i, pms_i, dpxy);
            }
        }
        let (_, n, pms_n, dpxy) = best_index_d;
        if n == 0 {
            return None;
        }

        self.used_dpxy.push(dpxy);
        self.used_pairs.insert((pms_0, pms_n));
        self.used_pairs.insert((pms_n, pms_0));
        self.pts[0].use_count += 1;
        self.pts[n].use_count += 1;

        self.pts.rotate_left(1);

        Some((pms_0, pms_n))
    }
}

//a PointMappingSet
//tp PointMappingSet
#[derive(Debug, Default)]
pub struct PointMappingSet {
    mappings: Vec<PointMapping>,
}

//ip Serialize for PointMappingSet
impl Serialize for PointMappingSet {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.mappings.len()))?;
        let sorted_order = self.sorted_order();
        for i in sorted_order {
            seq.serialize_element(&self.mappings[i])?;
        }
        seq.end()
    }
}

//ip Deserialize for PointMappingSet
impl<'de> Deserialize<'de> for PointMappingSet {
    fn deserialize<DE>(deserializer: DE) -> std::result::Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let mappings = Vec::<PointMapping>::deserialize(deserializer)?;
        Ok(Self { mappings })
    }
}

//ip PointMappingSet - constructors, add, remove
impl PointMappingSet {
    //fp new
    pub fn new() -> Self {
        Self::default()
    }

    //mp add_mappings
    pub fn add_mappings(&mut self, nps: &NamedPointSet, data: &[(&str, isize, isize)]) {
        for (name, px, py) in data {
            self.add_mapping(nps, name, &[*px as f64, *py as f64].into(), 5.0);
        }
    }

    //mp add_mapping
    pub fn add_mapping(
        &mut self,
        nps: &NamedPointSet,
        name: &str,
        screen: &Point2D,
        error: f64,
    ) -> bool {
        if let Some(model) = nps.get_pt(name) {
            self.mappings
                .push(PointMapping::new_npt(model, screen, error));
            true
        } else {
            false
        }
    }

    //mp remove_mapping
    pub fn remove_mapping(&mut self, n: usize) -> bool {
        if n < self.mappings.len() {
            self.mappings.remove(n);
            true
        } else {
            false
        }
    }

    //mp merge
    pub fn merge(&mut self, mut other: PointMappingSet) {
        self.mappings.append(&mut other.mappings);
    }

    //mp rebuild_with_named_point_set
    /// This rebuilds the point mapping set by *removing* the entries
    /// that are not in the named point set
    ///
    /// Used when rebuilding from a Json, here and by the Cip
    pub fn rebuild_with_named_point_set(&mut self, nps: &NamedPointSet) -> Vec<PointMapping> {
        let mut unmapped = vec![];
        let mut remove = vec![];
        for (n, p) in self.mappings.iter_mut().enumerate() {
            if let Some(np) = nps.get_pt(p.name()) {
                p.set_np(np.clone());
            } else {
                unmapped.push(p.name());
                remove.push(n);
            }
        }

        let mut result = vec![];
        for i in remove.into_iter().rev() {
            result.push(self.mappings.remove(i));
        }
        result
    }
}

//ip PointMappingSet - accessors
impl PointMappingSet {
    //ap len
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    //ap is_empty
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    //ap mappings
    pub fn mappings(&self) -> &[PointMapping] {
        &self.mappings
    }

    //ap mapping_of_np
    pub fn mapping_of_np(&self, np: &Rc<NamedPoint>) -> Option<&PointMapping> {
        let s = self
            .mappings
            .iter()
            .find(|pm| Rc::ptr_eq(np, pm.named_point()));
        s
    }

    //mp get_screen_pts
    pub fn get_screen_pts(&self) -> Vec<Point2D> {
        self.mappings.iter().map(|x| *x.screen()).collect()
    }
}

//ip PointMappingSet - Json
impl PointMappingSet {
    //mp read_json
    pub fn read_json(
        &mut self,
        nps: &NamedPointSet,
        toml: &str,
        allow_not_found: bool,
    ) -> Result<String> {
        let (pms, nf) = Self::from_json(nps, toml)?;
        if !allow_not_found && !nf.is_empty() {
            Err(Error::Msg(nf))
        } else {
            self.merge(pms);
            Ok(nf)
        }
    }

    //cp from_json
    pub fn from_json(nps: &NamedPointSet, json: &str) -> Result<(Self, String)> {
        let mut pms: Self = json::from_json("point map set", json)?;
        let pms_not_found = pms.rebuild_with_named_point_set(nps);
        if pms_not_found.is_empty() {
            Ok((pms, "".into()))
        } else {
            let mut r = String::new();
            let mut sep = "";
            for pms_nf in pms_not_found {
                r.push_str(&format!("{sep}'{}'", pms_nf.name()));
                sep = ", ";
            }
            Ok((
                pms,
                format!("Failed to find points {r} to map in named point set"),
            ))
        }
    }

    //mp to_json
    pub fn to_json(&self, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
    }

    //mp sorted_order
    pub fn sorted_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.mappings.len()).collect();
        order.sort_by(|a, b| self.mappings[*a].cmp(&self.mappings[*b]));
        order
    }
}

//ip PointMappingSet - Operations
impl PointMappingSet {
    //mi get_pxy_cog
    pub fn get_pxy_cog(&self) -> Point2D {
        let divider = self.mappings.len().min(1) as f64;
        let cog = Point2D::default();
        self.mappings.iter().fold(cog, |acc, m| acc + m.screen()) / divider
    }

    //mp get_good_screen_pairs
    pub fn get_good_screen_pairs<F>(&self, max_pairs: usize, filter: F) -> Vec<(usize, usize)>
    where
        F: Fn(usize, &PointMapping) -> bool,
    {
        let cog = self.get_pxy_cog();
        let mut good_screen_pairs = PmsGoodScreenPairSet::default();
        for (i, pms) in self
            .mappings
            .iter()
            .enumerate()
            .filter(|(_n, m)| m.is_mapped())
            .filter(|(n, m)| filter(*n, m))
        {
            good_screen_pairs.add_pt(i, &cog, pms);
        }
        good_screen_pairs.sort_pts().take(max_pairs).collect()
    }

    //mp add_good_model_lines
    pub fn add_good_model_lines<C, F>(&self, mls: &mut ModelLineSet<C>, filter: F, max_pairs: usize)
    where
        F: Fn(usize, &PointMapping) -> bool,
        C: CameraProjection,
    {
        for (i, j) in self.get_good_screen_pairs(max_pairs, filter) {
            mls.add_line((&self.mappings[i], &self.mappings[j]));
        }
    }
    //mp find_worst_error
    //
    // used by get_best_location
    //
    // worst_error returns just the error value
    pub fn find_worst_error<C: CameraProjection>(&self, camera: &C) -> (usize, f64) {
        let mut n = 0;
        let mut worst_e = 0.;
        for (i, pm) in self.mappings.iter().enumerate() {
            let e = pm.get_mapped_dpxy_error2(camera);
            if e > worst_e {
                n = i;
                worst_e = e;
            }
        }
        (n, worst_e)
    }

    //fp total_error
    // used by get_best_location
    //
    pub fn total_error<C: CameraProjection>(&self, camera: &C) -> f64 {
        self.mappings
            .iter()
            .fold(0.0, |acc, pm| acc + pm.get_mapped_dpxy_error2(camera))
    }

    //mp iter_mapped_rays
    pub fn iter_mapped_rays<C: CameraProjection>(
        &self,
        camera: &C,
        from_camera: bool,
    ) -> impl Iterator<Item = (&PointMapping, Ray)> {
        self.mappings()
            .iter()
            .map(move |pm| (pm, pm.get_mapped_ray(camera, from_camera)))
    }
}

//ip PointMappingSet - Camera locate and orient
impl PointMappingSet {
    //fp orient_camera_using_model_directions
    pub fn orient_camera_using_model_directions<C, F>(
        &self,
        camera: &mut C,
        filter: F,
    ) -> Result<f64>
    where
        F: Clone + Fn(usize, &PointMapping) -> bool,
        C: CameraProjection,
    {
        let mut qs = vec![];

        for (i, pm_i) in self
            .mappings
            .iter()
            .enumerate()
            .filter(|(_n, pm)| pm.is_mapped())
            .filter(|(n, pm)| filter.clone()(*n, pm))
        {
            let di_c = pm_i.get_mapped_unit_vector(camera);
            let di_m = (camera.position() - pm_i.model()).normalize();

            for (_, pm_j) in self
                .mappings
                .iter()
                .enumerate()
                .filter(|(j, _)| i != *j)
                .filter(|(_n, pm)| pm.is_mapped())
                .filter(|(n, pm)| filter.clone()(*n, pm))
            {
                let dj_c = pm_j.get_mapped_unit_vector(camera);
                let dj_m = (camera.position() - pm_j.model()).normalize();

                qs.push((
                    1.0,
                    utils::orientation_mapping_vpair_to_ppair(
                        di_m.as_ref(),
                        dj_m.as_ref(),
                        &di_c,
                        &dj_c,
                    )
                    .into(),
                ));
            }
        }
        if qs.is_empty() {
            return Err("No point mappings available to orient camera"
                .to_string()
                .into());
        }

        let (qr, e) = utils::weighted_average_many_with_err(&qs);
        camera.set_orientation(&qr);
        let te = self.total_error(camera);
        eprintln!("Error in qr's {e} total error {te} QR: {qr}q");
        Ok(te)
    }
}
