//a Imports
use std::collections::HashMap;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use super::{Point2D, Point3D};

//a NamedPoint
//tp NamedPoint
/// A point in model space, with a name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedPoint {
    name: String,
    /// The 3D model coordinate this point corresponds to
    ///
    /// This is known for a calibration point!
    model: Point3D,
}

//ip NamedPoint
impl NamedPoint {
    pub fn new<S: Into<String>>(name: S, model: Point3D) -> Self {
        let name = name.into();
        Self { name, model }
    }
    pub fn model(&self) -> &Point3D {
        &self.model
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}
//a NamedPointSet
//tp NamedPointSet
#[derive(Debug, Default)]
pub struct NamedPointSet {
    points: HashMap<String, Rc<NamedPoint>>,
}

//ip NamedPointSet
impl NamedPointSet {
    //fp new
    pub fn new() -> Self {
        Self::default()
    }

    //fp from_json
    pub fn from_json(toml: &str) -> Result<Self, String> {
        serde_json::from_str(toml).map_err(|e| {
            format!(
                "Error in parsing json at line {} column {}: {}",
                e.line(),
                e.column(),
                e
            )
        })
    }

    //fp to_json
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("{}", e))
    }

    //fp add_set
    pub fn add_set(&mut self, data: &[(&str, [f64; 3])]) {
        for (name, pt) in data {
            self.add_pt(*name, (*pt).into());
        }
    }

    //fp add_pt
    pub fn add_pt<S: Into<String>>(&mut self, name: S, model: Point3D) {
        let name = name.into();
        let pt = Rc::new(NamedPoint::new(name.clone(), model));
        self.points.insert(name, pt);
    }

    //fp get_pt
    pub fn get_pt(&self, name: &str) -> Option<Rc<NamedPoint>> {
        self.points.get(name).map(|a| a.clone())
    }
    //fp iter
    pub fn iter(&self) -> std::collections::hash_map::Iter<String, Rc<NamedPoint>> {
        self.points.iter()
    }
}

//ft test_json
#[test]
fn test_json() -> Result<(), String> {
    let mut nps = NamedPointSet::default();
    nps.add_pt("fred", [1., 2., 3.].into());
    let s = nps.to_json()?;
    assert_eq!(s, r#"[["fred",[1.0,2.0,3.0]]]"#);
    let nps = NamedPointSet::from_json(
        r#"
[["fred", [1, 2, 3]]]
"#,
    )?;
    assert!(nps.get_pt("jim").is_none(), "Jim is not a point");
    assert!(nps.get_pt("fred").is_some(), "Fred is a point");
    assert_eq!(nps.get_pt("fred").unwrap().model()[0], 1.0);
    assert_eq!(nps.get_pt("fred").unwrap().model()[1], 2.0);
    assert_eq!(nps.get_pt("fred").unwrap().model()[2], 3.0);
    Ok(())
}

//ip Serialize for NamedPointSet
impl Serialize for NamedPointSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.points.len()))?;
        for (name, pt) in self.points.iter() {
            seq.serialize_element(&(name, pt.model()))?;
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
        let array = Vec::<(String, Point3D)>::deserialize(deserializer)?;
        let mut nps = NamedPointSet::default();
        for (name, model) in array {
            nps.add_pt(name, model);
        }
        Ok(nps)
    }
}

//a PointMappingSet
//ip PointMappingSet
#[derive(Default, Serialize, Deserialize)]
pub struct PointMappingSet {
    mappings: Vec<PointMapping>,
}

//ip PointMappingSet
impl PointMappingSet {
    //fp new
    pub fn new() -> Self {
        Self::default()
    }

    //mp rebuild_with_named_point_set
    pub fn rebuild_with_named_point_set(&mut self, nps: &NamedPointSet) -> Vec<String> {
        let mut unmapped = Vec::new();
        for p in self.mappings.iter_mut() {
            if !p.within_named_point_set(nps) {
                unmapped.push(p.name().into());
            }
        }
        unmapped
    }

    //mp add_mappings
    pub fn add_mappings(&mut self, nps: &NamedPointSet, data: &[(&str, isize, isize)]) {
        for (name, px, py) in data {
            self.add_mapping(nps, name, &[*px as f64, *py as f64].into());
        }
    }

    //mp add_mapping
    pub fn add_mapping(&mut self, nps: &NamedPointSet, name: &str, screen: &Point2D) -> bool {
        if let Some(model) = nps.get_pt(name) {
            self.mappings.push(PointMapping::new_npt(model, screen));
            true
        } else {
            false
        }
    }
    //ap mappings
    pub fn mappings(&self) -> &[PointMapping] {
        &self.mappings
    }
}

//a PointMapping
//tp PointMapping
#[derive(Debug, Serialize, Deserialize)]
pub struct PointMapping {
    /// The 3D model coordinate this point corresponds to
    ///
    /// This is known for a calibration point!
    pub model: Rc<NamedPoint>,
    /// Screen coordinate
    pub screen: Point2D,
}

//ip PointMapping
impl PointMapping {
    //fp new_npt
    pub fn new_npt(model: Rc<NamedPoint>, screen: &Point2D) -> Self {
        PointMapping {
            model,
            screen: *screen,
        }
    }

    //fp within_named_point_set
    pub fn within_named_point_set(&mut self, nps: &NamedPointSet) -> bool {
        if let Some(model) = nps.get_pt(self.model.name()) {
            self.model = model;
            true
        } else {
            false
        }
    }

    //fp new
    pub fn new(model: &Point3D, screen: &Point2D) -> Self {
        let model = Rc::new(NamedPoint::new("unnamed", *model));
        Self::new_npt(model, screen)
    }

    //mp model
    pub fn model(&self) -> &Point3D {
        self.model.model()
    }

    //mp screen
    pub fn screen(&self) -> &Point2D {
        &self.screen
    }

    //mp name
    pub fn name(&self) -> &str {
        self.model.name()
    }

    //zz All done
}
