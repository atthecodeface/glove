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
    /// The 3D model coordinate this point corresponds to and the radius of uncertainty
    ///
    /// This is known for a calibration point, with 0 uncertainty!
    ///
    /// The units are mm (as that is what cameras focal lengths are in)
    // #[serde(deserialize_with = "deserialize_model")]
    model: RefCell<Option<(Point3D, f64)>>,
}

#[allow(dead_code)]
fn deserialize_model<'de, D>(deserializer: D) -> Result<RefCell<Option<(Point3D, f64)>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // let model = <Option<(Point3D, f64)>>::deserialize(deserializer)?;
    let model = <Option<Point3D>>::deserialize(deserializer)?;
    let model = model.map(|a| (a, 0.));
    Ok(model.into())
}

//ip PartialEq for NamedPoint
impl PartialEq for NamedPoint {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

//ip Eq for NamedPoint
impl Eq for NamedPoint {}

//ip Ord for NamedPoint
impl Ord for NamedPoint {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

//ip PartialOrd for NamedPoint
impl PartialOrd for NamedPoint {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

//ip NamedPoint
impl NamedPoint {
    pub fn new<S: Into<String>>(name: S, color: Color, model: Option<(Point3D, f64)>) -> Self {
        let name = name.into();
        let model = model.into();
        Self { name, color, model }
    }
    #[inline]
    pub fn is_unmapped(&self) -> bool {
        self.model.borrow().is_none()
    }
    #[inline]
    pub fn model(&self) -> (Point3D, f64) {
        (*self.model.borrow()).unwrap_or_default()
    }
    #[inline]
    pub fn opt_model(&self) -> Option<(Point3D, f64)> {
        *self.model.borrow()
    }
    #[inline]
    pub fn color(&self) -> &Color {
        &self.color
    }
    #[inline]
    pub fn set_model(&self, model: Option<(Point3D, f64)>) {
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

    //mp sorted_order
    pub fn sorted_order(&self) -> Vec<usize> {
        let mut order: Vec<usize> = (0..self.names.len()).collect();
        order.sort_by(|a, b| self.names[*a].cmp(&self.names[*b]));
        order
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
                self.add_np(np);
            }
        }
    }
    //mp add_np
    /// Requires np to not be in the name set already
    pub fn add_np(&mut self, np: &NamedPoint) {
        let opt_model = np.opt_model();
        let err = opt_model.map_or(0.0, |m| m.1);
        let opt_model = opt_model.map(|m| m.0);
        self.add_pt(np.name.clone(), np.color, opt_model, err);
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
        let sorted_order = self.sorted_order();
        for i in sorted_order {
            let name = &self.names[i];
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
        let mut nps = NamedPointSet::default();
        let array = Vec::<NamedPoint>::deserialize(deserializer)?;
        for np in array {
            nps.add_np(&np);
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
    nps.add_pt("fred", c, Some([1., 2., 3.].into()), 0.0);
    let s = nps.to_json()?;
    assert_eq!(s, r##"[["fred","#000000",[1.0,2.0,3.0]]]"##);
    let nps = NamedPointSet::from_json(
        r##"
[["fred", "#000000", [1, 2, 3]]]
"##,
    )?;
    assert!(nps.get_pt("jim").is_none(), "Jim is not a point");
    assert!(nps.get_pt("fred").is_some(), "Fred is a point");
    assert_eq!(nps.get_pt("fred").unwrap().model().0[0], 1.0);
    assert_eq!(nps.get_pt("fred").unwrap().model().0[1], 2.0);
    assert_eq!(nps.get_pt("fred").unwrap().model().0[2], 3.0);
    Ok(())
}
