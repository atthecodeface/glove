//a Imports
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use ic_base::{Error, JsonParsable, Point3D, Result, Tag, TagMap, TagSet};
use ic_camera::CameraProjection;
use ic_image::Color8;

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

impl NamedPointSet {
    /// Get the number of points in the [NamedPointSet]
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Return true if the set is empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// This must be invoked with the creation of a NamedPointSet
    ///
    /// The TagSet may be empty; when the NamedPointSet is deserialized its tags
    /// are set to be Owned, and so merging with another TagSet is fine
    pub fn set_tag_set(&mut self, tags: Rc<TagSet>) {
        self.points.set_tag_set(tags)
    }

    /// Create JSON for the set
    ///
    /// The JSON is the 'owner' of the names of the points; other types that are
    /// serialized using the names of points do so by reference, the NPS does it
    /// by ownership
    pub fn to_json(&self, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
    }

    /// Merge another NPS into this one
    pub fn merge(&mut self, other: Self) {
        for other_np in other.points.into_values() {
            if let Some(self_np) = self.points.get_tag(&other_np.ref_tag()) {
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

    /// Return true if the name of the named point presented is present in the set
    pub fn has_np(&self, np: &NamedPoint) -> bool {
        self.points.has_name(np.ref_tag().as_str())
    }

    /// Add a named point to the set; if the name of the point is already
    /// present then the current contents are returned (i.e. the old point)
    pub fn add_np(&mut self, np: NamedPoint) -> Option<Rc<NamedPoint>> {
        self.points.add_data(np)
    }

    /// Add a point to the named point set
    ///
    /// This must happen only after the TagSet is defined
    pub fn add_pt(
        &mut self,
        name: &str,
        color: Color8,
        at_infinity: bool,
        model: Option<Point3D>,
        err: f64,
    ) -> Option<Rc<NamedPoint>> {
        let tag = Tag::owned(name);
        let model = model.map(|m| (at_infinity, m, err));
        self.add_np(NamedPoint::new(tag, color, model))
    }

    /// Create a vector of the named points with a specific color
    pub fn of_color(&self, color: &Color8) -> Vec<Rc<NamedPoint>> {
        self.points
            .iter()
            .filter(|v| color.color_eq(&v.color()))
            .cloned()
            .collect()
    }

    pub fn resolve_pt(&self, new_np: &NamedPoint) -> Option<Rc<NamedPoint>> {
        self.points.get_data(new_np.ref_tag().as_str()).cloned()
    }

    pub fn get_rc_np(&self, name: &str) -> Option<Rc<NamedPoint>> {
        self.points.get_data(name).cloned()
    }

    /// Get the number of *other* users of a named point
    pub fn pt_use_count(&self, name: &str) -> usize {
        if !self.points.has_name(name) {
            0
        } else {
            self.points.get_tag_use_count(name).unwrap() - 1
        }
    }

    /// Remove the named point from the set, by name
    pub fn remove_pt(&mut self, name: &str) -> Option<Rc<NamedPoint>> {
        self.points.remove_data(name)
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
                let color = Color8::try_from(np)?;
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

            let name = np.ref_tag();
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
