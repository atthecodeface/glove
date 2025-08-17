//a Imports
use std::collections::HashMap;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use ic_base::{json, Error, Point3D, Result, Tag, TagSet};
use ic_camera::CameraProjection;
use ic_image::Color;

use crate::NamedPoint;

//a NamedPointSet
//tp NamedPointSet
#[derive(Debug, Default)]
pub struct NamedPointSet {
    points: HashMap<Tag, Rc<NamedPoint>>,
    /// The tags, so that tags can be resolved to Shared without
    /// resorting to any other struct
    ///
    /// This is an Rc so the same TagSet can be shared
    ///
    /// It is immutable as the TagSet has interior mutability
    tags: Rc<TagSet>,
}

//ip Serialize for NamedPointSet
impl Serialize for NamedPointSet {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let sorted_order = self.sorted_order();
        let mut seq = serializer.serialize_seq(Some(sorted_order.len()))?;
        for name in sorted_order {
            let np = self.points.get(&name).unwrap();
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
            nps.add_np(np);
        }
        Ok(nps)
    }
}

//ip NamedPointSet
impl NamedPointSet {
    //fp set_tag_set
    pub fn set_tag_set(&mut self, tags: Rc<TagSet>) {
        self.tags = tags;
        let old_points = std::mem::take(&mut self.points);
        for (t, np) in old_points.into_iter() {
            assert!(!t.is_resolved());
            let t = self.tags.resolve_tag(t);
            self.points.insert(t, np);
        }
    }

    //mp sorted_order
    pub fn sorted_order(&self) -> Vec<Tag> {
        let mut order: Vec<_> = self.points.keys().map(|s| s.clone()).collect();
        order.sort_by(|a, b| a.cmp(&b));
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
    ///
    /// The other NPS cannot be in use
    pub fn merge(&mut self, other: Self) {
        for np in other.points.into_values() {
            let np_name = np.name();
            if self.points.contains_key(np_name) {
                if !np.is_unmapped() && self.points.get_mut(np_name).unwrap().is_unmapped() {
                    self.points
                        .get_mut(np_name)
                        .unwrap()
                        .set_model(Some(np.model()));
                }
            } else {
                self.add_np(Rc::into_inner(np).unwrap()); // This will fail if the named point is in use in a PMS
            }
        }
    }

    //fp has_np
    pub fn has_np(&self, np: &NamedPoint) -> bool {
        self.points.contains_key(np.name())
    }

    //mp add_np
    /// Requires np to not be in the name set already
    pub fn add_np(&mut self, mut np: NamedPoint) {
        np.resolve_name(&self.tags);
        let name = np.name().clone();
        assert!(self.points.insert(name, Rc::new(np)).is_none());
    }

    //fp add_pt
    pub fn add_pt<S: Into<Tag>>(
        &mut self,
        name: S,
        color: Color,
        model: Option<Point3D>,
        err: f64,
    ) {
        let model = model.map(|m| (m, err));
        self.add_np(NamedPoint::new(name.into(), color, model));
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
    pub fn iter(&self) -> std::collections::hash_map::Iter<Tag, Rc<NamedPoint>> {
        self.points.iter()
    }

    //dp into_iter
    pub fn into_iter(self) -> impl Iterator<Item = Option<NamedPoint>> {
        self.points.into_values().map(|p| Rc::into_inner(p))
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
