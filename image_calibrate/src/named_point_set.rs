//a Imports
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::Point3D;

//a NamedPoint
//tp NamedPoint
/// A point in model space, with a name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedPoint {
    name: String,
    /// The 3D model coordinate this point corresponds to
    ///
    /// This is known for a calibration point!
    model: RefCell<Point3D>,
}

//ip NamedPoint
impl NamedPoint {
    pub fn new<S: Into<String>>(name: S, model: Point3D) -> Self {
        let name = name.into();
        let model = model.into();
        Self { name, model }
    }
    #[inline]
    pub fn model(&self) -> Point3D {
        *self.model.borrow()
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
