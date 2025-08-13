//a Imports
use std::collections::HashMap;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use ic_base::{json, Error, Point3D, Result};
use ic_camera::CameraProjection;
use ic_image::Color;

use crate::NamedPoint;

//a NamedPointSet
//tp NamedPointSet
#[derive(Debug, Default)]
pub struct NamedPointSet {
    points: HashMap<String, Rc<NamedPoint>>,
    names: Vec<String>,
}

//ip Serialize for NamedPointSet
impl Serialize for NamedPointSet {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.names.len()))?;
        let sorted_order = self.sorted_order();
        for i in sorted_order {
            let name = &self.names[i];
            let np = self.points.get(name).unwrap();
            let color = np.color();
            let model = np.opt_model();
            seq.serialize_element(&(name, color, model))?;
        }
        seq.end()
    }
}

//ip Deserialize for NamedPointSet
impl<'de> Deserialize<'de> for NamedPointSet {
    fn deserialize<DE>(deserializer: DE) -> std::result::Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let mut nps = NamedPointSet::default();
        let array = Vec::<NamedPoint>::deserialize(deserializer)?;
        for np in array {
            nps.add_np(&np);
        }
        Ok(nps)
    }
}

//ip NamedPointSet
impl NamedPointSet {
    //fp new
    pub fn new() -> Self {
        Self::default()
    }

    //mp sorted_order
    pub fn sorted_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.names.len()).collect();
        order.sort_by(|a, b| self.names[*a].cmp(&self.names[*b]));
        order
    }

    //fp from_json
    pub fn from_json(json: &str) -> Result<Self> {
        json::from_json("named point set", json)
    }

    //mp to_json
    pub fn to_json(&self, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
    }

    //mp merge
    /// Merge another NPS into this one
    pub fn merge(&mut self, other: &Self) {
        for np in other.points.values() {
            let np_name = np.name();
            if self.points.contains_key(np_name) {
                if !np.is_unmapped() && self.points.get_mut(np_name).unwrap().is_unmapped() {
                    self.points
                        .get_mut(np_name)
                        .unwrap()
                        .set_model(Some(np.model()));
                }
            } else {
                self.add_np(np);
            }
        }
    }

    //fp has_np
    pub fn has_np(&self, np: &NamedPoint) -> bool {
        self.points.contains_key(np.name())
    }

    //mp add_np
    /// Requires np to not be in the name set already
    pub fn add_np(&mut self, np: &NamedPoint) {
        let opt_model = np.opt_model();
        let err = opt_model.map_or(0.0, |m| m.1);
        let opt_model = opt_model.map(|m| m.0);
        self.add_pt(np.name(), *np.color(), opt_model, err);
    }

    //fp add_pt
    pub fn add_pt<S: Into<String>>(
        &mut self,
        name: S,
        color: Color,
        model: Option<Point3D>,
        err: f64,
    ) {
        let name = name.into();
        let model = model.map(|m| (m, err));
        let pt = Rc::new(NamedPoint::new(name.clone(), color, model));
        if self.points.insert(name.clone(), pt).is_none() {
            self.names.push(name);
        }
    }

    //fp of_color
    pub fn of_color(&self, color: &Color) -> Vec<Rc<NamedPoint>> {
        self.points
            .values()
            .filter(|v| color.color_eq(v.color()))
            .cloned()
            .collect()
    }

    //fp get_pt
    pub fn get_pt(&self, name: &str) -> Option<Rc<NamedPoint>> {
        self.points.get(name).cloned()
    }

    //fp get_pt_err
    pub fn get_pt_err(&self, name: &str) -> Result<Rc<NamedPoint>> {
        self.get_pt(name).ok_or_else(|| {
            Error::Database(format!("Named point set does not contain name '{name}'"))
        })
    }

    //fp iter
    pub fn iter(&self) -> std::collections::hash_map::Iter<String, Rc<NamedPoint>> {
        self.points.iter()
    }
}

//ip NamedPointSet - show
impl NamedPointSet {
    //fp show_mappings
    pub fn show_mappings<C: CameraProjection>(&self, camera: &C) {
        for (name, np) in &self.points {
            if np.is_unmapped() {
                continue;
            }

            let (model, error) = np.model();
            let camera_pxy = camera.world_xyz_to_px_abs_xy(&model);

            println!("{name} : {model}+-{error} maps to {camera_pxy}",);
        }
    }
}
