//a Imports
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{json, Color, Point3D};

//a NamedPoint
//tp NamedPoint
/// A point in model space, with a name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedPoint {
    /// Name of the point
    name: String,
    /// Color of the point in calibration images
    color: Color,
    /// The 3D model coordinate this point corresponds to
    ///
    /// This is known for a calibration point!
    model: RefCell<Option<Point3D>>,
}

//ip NamedPoint
impl NamedPoint {
    pub fn new<S: Into<String>>(name: S, color: Color, model: Option<Point3D>) -> Self {
        let name = name.into();
        let model = model.into();
        Self { name, color, model }
    }
    #[inline]
    pub fn is_unmapped(&self) -> bool {
        self.model.borrow().is_none()
    }
    #[inline]
    pub fn model(&self) -> Point3D {
        (*self.model.borrow()).unwrap_or_default()
    }
    #[inline]
    pub fn opt_model(&self) -> Option<Point3D> {
        *self.model.borrow()
    }
    #[inline]
    pub fn color(&self) -> &Color {
        &self.color
    }
    #[inline]
    pub fn set_model(&self, model: Option<Point3D>) {
        *self.model.borrow_mut() = model;
    }
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
}
//a NamedPointSet
//tp NamedPointSet
#[derive(Debug, Default)]
pub struct NamedPointSet {
    points: HashMap<String, Rc<NamedPoint>>,
    names: Vec<String>,
}

//ip NamedPointSet
impl NamedPointSet {
    //fp new
    pub fn new() -> Self {
        Self::default()
    }

    //fp from_json
    pub fn from_json(json: &str) -> Result<Self, String> {
        json::from_json("named point set", json)
    }

    //fp to_json
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("{}", e))
    }

    //mp merge
    /// Merge another NPS into this one
    pub fn merge(&mut self, other: &Self) {
        for np in other.points.values() {
            if self.points.contains_key(&np.name) {
                if !np.is_unmapped() && self.points.get_mut(&np.name).unwrap().is_unmapped() {
                    self.points
                        .get_mut(&np.name)
                        .unwrap()
                        .set_model(Some(np.model()));
                }
            } else {
                self.add_pt(np.name.clone(), np.color, np.opt_model());
            }
        }
    }

    //fp add_pt
    pub fn add_pt<S: Into<String>>(&mut self, name: S, color: Color, model: Option<Point3D>) {
        let name = name.into();
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
    pub fn get_pt_err(&self, name: &str) -> Result<Rc<NamedPoint>, String> {
        self.get_pt(name)
            .ok_or_else(|| format!("Named point set does not contain name '{}'", name))
    }

    //fp iter
    pub fn iter(&self) -> std::collections::hash_map::Iter<String, Rc<NamedPoint>> {
        self.points.iter()
    }
}

//ip Serialize for NamedPointSet
impl Serialize for NamedPointSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.names.len()))?;
        for name in self.names.iter() {
            let np = self.points.get(name).unwrap();
            let color = np.color();
            let model = np.model();
            seq.serialize_element(&(name, color, model))?;
        }
        seq.end()
    }
}

//ip Deserialize for NamedPointSet
impl<'de> Deserialize<'de> for NamedPointSet {
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let array = Vec::<(String, Color, Option<Point3D>)>::deserialize(deserializer)?;
        let mut nps = NamedPointSet::default();
        for (name, color, model) in array {
            nps.add_pt(name, color, model);
        }
        Ok(nps)
    }
}

//a Tests
//ft test_json_0
#[test]
fn test_json_0() -> Result<(), String> {
    let c = Color::black();
    let mut nps = NamedPointSet::default();
    nps.add_pt("fred", c, Some([1., 2., 3.].into()));
    let s = nps.to_json()?;
    assert_eq!(s, r##"[["fred","#000000",[1.0,2.0,3.0]]]"##);
    let nps = NamedPointSet::from_json(
        r##"
[["fred", "#000000", [1, 2, 3]]]
"##,
    )?;
    assert!(nps.get_pt("jim").is_none(), "Jim is not a point");
    assert!(nps.get_pt("fred").is_some(), "Fred is a point");
    assert_eq!(nps.get_pt("fred").unwrap().model()[0], 1.0);
    assert_eq!(nps.get_pt("fred").unwrap().model()[1], 2.0);
    assert_eq!(nps.get_pt("fred").unwrap().model()[2], 3.0);
    Ok(())
}
