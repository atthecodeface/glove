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
    names: Vec<String>,
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
        if self.points.insert(name.clone(), pt).is_none() {
            self.names.push(name);
        }
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

//ip Serialize for NamedPointSet
impl Serialize for NamedPointSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.names.len()))?;
        for name in self.names.iter() {
            let model = self.points.get(name).unwrap().model();
            seq.serialize_element(&(name, model))?;
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
//tp PointMappingSet
#[derive(Default)]
pub struct PointMappingSet {
    mappings: Vec<PointMapping>,
}

//ip Serialize for PointMappingSet
impl Serialize for PointMappingSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.mappings.len()))?;
        for e in self.mappings.iter() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

//ip Deserialize for PointMappingSet
impl<'de> Deserialize<'de> for PointMappingSet {
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let mappings = Vec::<PointMapping>::deserialize(deserializer)?;
        Ok(Self { mappings })
    }
}

//ip PointMappingSet
impl PointMappingSet {
    //fp new
    pub fn new() -> Self {
        Self::default()
    }

    //mp merge
    pub fn merge(&mut self, mut other: PointMappingSet) {
        self.mappings.append(&mut other.mappings);
    }

    //mp read_json
    pub fn read_json(&mut self, nps: &NamedPointSet, toml: &str) -> Result<(), String> {
        let pms = Self::from_json(nps, toml)?;
        self.merge(pms);
        Ok(())
    }

    //fp from_json
    pub fn from_json(nps: &NamedPointSet, toml: &str) -> Result<Self, String> {
        let mut pms: Self = serde_json::from_str(toml).map_err(|e| {
            format!(
                "Error in parsing json at line {} column {}: {}",
                e.line(),
                e.column(),
                e
            )
        })?;
        let errs = pms.rebuild_with_named_point_set(nps);
        if errs.is_empty() {
            Ok(pms)
        } else {
            let mut r = String::new();
            let mut sep = "";
            for e in errs {
                r.push_str(&format!("{}'{}'", sep, e));
                sep = ", ";
            }
            Err(format!(
                "Failed to find points {} to map in named point set",
                r
            ))
        }
    }

    //fp to_json
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("{}", e))
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
#[derive(Debug)]
pub struct PointMapping {
    /// The 3D model coordinate this point corresponds to
    ///
    /// This is known for a calibration point!
    pub model: Rc<NamedPoint>,
    /// Screen coordinate
    pub screen: Point2D,
}

//ip Serialize for PointMapping
impl Serialize for PointMapping {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeTuple;
        let mut seq = serializer.serialize_tuple(2)?;
        seq.serialize_element(self.model.name())?;
        seq.serialize_element(&self.screen)?;
        seq.end()
    }
}

//ip Deserialize for PointMapping
impl<'de> Deserialize<'de> for PointMapping {
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let (model_name, screen) = <(String, Point2D)>::deserialize(deserializer)?;
        let model = Rc::new(NamedPoint::new(model_name, [0., 0., 0.].into()));
        Ok(Self { model, screen })
    }
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
//a Tests
//ft test_json_0
#[test]
fn test_json_0() -> Result<(), String> {
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

//ft test_json_1
#[test]
fn test_json_1() -> Result<(), String> {
    let mut nps = NamedPointSet::default();
    nps.add_pt("fred", [1., 2., 3.].into());
    let mut pms = PointMappingSet::new();
    pms.add_mapping(&nps, "fred", &[1., 2.].into());
    let s = pms.to_json()?;
    assert_eq!(s, r#"[["fred",[1.0,2.0]]]"#);
    let pms = PointMappingSet::from_json(
        &nps,
        r#"
[["fred", [1, 2]]]
"#,
    )?;
    assert_eq!(pms.mappings().len(), 1);
    assert_eq!(pms.mappings()[0].name(), "fred");
    assert_eq!(pms.mappings()[0].screen()[0], 1.0);
    assert_eq!(pms.mappings()[0].screen()[1], 2.0);
    Ok(())
}
