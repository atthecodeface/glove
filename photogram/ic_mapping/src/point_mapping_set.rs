//a Imports
use std::collections::HashSet;
use std::rc::Rc;

use geo_nd::Vector;
use serde::{Deserialize, Serialize};

use ic_base::{json, Error, Point2D, Point3D, Result};
use ic_image::Color;

use crate::{NamedPoint, NamedPointSet, PointMapping};

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
            if !p.within_named_point_set(nps) {
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
        self.mappings.iter().find(|pm| Rc::ptr_eq(np, &pm.model))
    }

    //mp get_screen_pts
    pub fn get_screen_pts(&self) -> Vec<Point2D> {
        self.mappings.iter().map(|x| x.screen).collect()
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
        self.mappings.iter().fold(cog, |acc, m| acc + m.screen) / divider
    }

    //mp get_good_screen_pairs
    pub fn get_good_screen_pairs<F>(&self, max_pairs: usize, filter: F) -> Vec<(usize, usize)>
    where
        F: Fn(&PointMapping) -> bool,
    {
        let cog = self.get_pxy_cog();

        // Get a list of the *mapped* points that pass the filter
        // sorted in order of distance from COG
        //
        // Store with it a use count that starts at 0
        let mut v: Vec<(usize, usize, f64, Point2D)> = self
            .mappings
            .iter()
            .enumerate()
            .filter(|(_n, m)| m.is_mapped())
            .filter(|(_n, m)| filter(m))
            .map(|(n, m)| {
                let pxy = m.screen - cog;
                (n, 0, pxy.length(), pxy)
            })
            .collect();
        v.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap().reverse());

        let mut pairs: Vec<(usize, usize)> = vec![];
        let mut used_pairs: HashSet<(usize, usize)> = HashSet::default();
        for i in 0..v.len() {
            let mut furthest_v = i;
            let mut max_d = 0.0;
            for j in 0..v.len() {
                if i == j {
                    continue;
                }
                if used_pairs.contains(&(i, j)) {
                    continue;
                }
                let d_ij = v[i].3 - v[j].3;
                let mut d = 0.0;
                if pairs.is_empty() {
                    d = d_ij.length() / ((2 + v[i].1 + v[j].1) as f64);
                } else {
                    for (p0, p1) in &pairs {
                        let d_ik = self.mappings[*p0].screen - self.mappings[*p1].screen;
                        d += (d_ij[0] * d_ik[1] - d_ij[1] * d_ik[0]).abs();
                    }
                }
                if d > max_d {
                    furthest_v = j;
                    max_d = d;
                }
            }
            if furthest_v == i {
                continue;
            }
            pairs.push((v[i].0, v[furthest_v].0));
            used_pairs.insert((i, furthest_v));
            used_pairs.insert((furthest_v, i));
            v[i].1 += 1;
            v[furthest_v].1 += 1;
        }

        // Debug
        let dgb: Vec<(String, String)> = pairs
            .iter()
            .map(|(x, y)| {
                (
                    self.mappings[*x].name().to_owned(),
                    self.mappings[*y].name().to_owned(),
                )
            })
            .collect();
        eprintln!("Pairs : {:?}", dgb);
        pairs
    }
}
