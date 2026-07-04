//a Imports
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use ic_base::{Error, JsonParsable, Point3D, Result, Tag, TagMap, TagSet};
use ic_camera::CameraProjection;
use ic_image::Color;

use crate::NamedPoint;

//a NamedPointSet
//tp NamedPointSet
#[derive(Debug, Default)]
pub struct NamedPointSet {
    points: TagMap<NamedPoint>,
}

//ip JsonParsable for NamedPointSet
impl JsonParsable for NamedPointSet {
    fn reason() -> &'static str {
        "named point set"
    }
    type PostParseArg = ();
    type PostParseResult = Self;
    fn post_parse(self, _: &()) -> Result<Self> {
        Ok(self)
    }
}

//ip Serialize for NamedPointSet
impl Serialize for NamedPointSet {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.points.serialize(serializer)
    }
}

//ip Deserialize for NamedPointSet
impl<'de> Deserialize<'de> for NamedPointSet {
    fn deserialize<DE>(deserializer: DE) -> std::result::Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let points = TagMap::deserialize(deserializer)?;
        Ok(Self { points })
    }
}

//ip NamedPointSet
impl NamedPointSet {
    //ap len
    pub fn len(&self) -> usize {
        self.points.len()
    }

    //ap is_empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    //fp set_tag_set
    pub fn set_tag_set(&mut self, tags: Rc<TagSet>) {
        self.points.set_tag_set(tags)
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
        for other_np in other.points.into_values() {
            if let Some(self_np) = self.points.get_data(other_np.name()) {
                let other_np_is_unmapped = other_np.is_unmapped();
                let self_np_is_unmapped = self_np.is_unmapped();
                if self_np_is_unmapped && !other_np_is_unmapped {
                    self_np.set_model(Some(other_np.model()));
                }
            } else if let Some(other_np) = Rc::into_inner(other_np) {
                // This will return None as the other is not in the named point set
                let _ = self.add_np(other_np);
            } else {
                panic!("Named point is being merged but is in use elsewhere");
            }
        }
    }

    //fp has_np
    pub fn has_np(&self, np: &NamedPoint) -> bool {
        self.points.has_tag(np.name())
    }

    //mp add_np
    /// Requires np to not be in the name set already
    pub fn add_np(&mut self, np: NamedPoint) -> Option<Rc<NamedPoint>> {
        self.points.add_data(np)
    }

    //mp add_pt
    /// Add a point to the named point set
    ///
    /// This must happen only after the TagSet is defined
    pub fn add_pt(
        &mut self,
        name: &str,
        color: Color,
        at_infinity: bool,
        model: Option<Point3D>,
        err: f64,
    ) -> Option<Rc<NamedPoint>> {
        let tag = Tag::owned(name);
        let model = model.map(|m| (at_infinity, m, err));
        self.add_np(NamedPoint::new(tag, color, model))
    }

    //fp of_color
    pub fn of_color(&self, color: &Color) -> Vec<Rc<NamedPoint>> {
        self.points
            .iter()
            .filter(|v| color.color_eq(v.color()))
            .cloned()
            .collect()
    }

    //fp get_pt
    pub fn get_pt(&self, name: &str) -> Option<Rc<NamedPoint>> {
        self.points.get_data(name).cloned()
    }

    //fp get_pt_err
    pub fn get_pt_err(&self, name: &str) -> Result<Rc<NamedPoint>> {
        self.points
            .get_data(name)
            .ok_or_else(|| {
                Error::Database(format!("Named point set does not contain name '{name}'"))
            })
            .cloned()
    }

    //mp select
    pub fn select<'a, I>(&self, search: I) -> Result<Vec<Rc<NamedPoint>>>
    where
        I: Iterator<Item = &'a str> + 'a,
    {
        let mut r = vec![];
        for np in search {
            if np.is_empty() {
                continue;
            }
            if np.as_bytes()[0] == b'#' {
                let color = Color::try_from(np)?;
                for np in self.of_color(&color) {
                    if !r.iter().any(|n| Rc::ptr_eq(n, &np)) {
                        r.push(np);
                    }
                }
            } else {
                r = self.points.fold_search(np, false, r, |mut r, np| {
                    if !r.iter().any(|n| Rc::ptr_eq(n, np)) {
                        r.push(np.clone());
                    }
                    r
                })?;
            }
        }
        Ok(r)
    }

    //fp iter
    pub fn iter(&self) -> impl Iterator<Item = &Rc<NamedPoint>> {
        self.points.iter()
    }

    //dp into_iter
    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(self) -> impl Iterator<Item = Option<NamedPoint>> {
        self.points.into_values().map(Rc::into_inner)
    }
}

//ip NamedPointSet - show
impl NamedPointSet {
    //fp show_mappings
    pub fn show_mappings<C: CameraProjection>(&self, camera: &C) {
        for np in self.points.iter() {
            if np.is_unmapped() {
                continue;
            }

            let name = np.name();
            let (at_infinity, model, error) = np.model();
            if at_infinity {
                if let Some(camera_pxy) = camera.world_dir_to_opt_px_abs_xy(&model) {
                    println!("{name} : {model} direction maps to {camera_pxy}");
                } else {
                    println!("{name} : {model} direction is behind camera",);
                }
            } else {
                let camera_pxy = camera.world_xyz_to_px_abs_xy(&model);
                println!("{name} : {model}+-{error} maps to {camera_pxy}");
            }
        }
    }
}
