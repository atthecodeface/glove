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
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NamedPointSet {
    points: HashMap<String, Rc<NamedPoint>>,
}

//ip NamedPointSet
impl NamedPointSet {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn from_toml(toml: &str) -> Result<Self, String> {
        toml::from_str(toml).map_err(|e| {
            format!(
                "Error in parsing toml at {:?}: {}",
                e.span().unwrap(),
                e.message()
            )
        })
    }
    pub fn add_set(&mut self, data: &[(&str, [f64; 3])]) {
        for (name, pt) in data {
            self.add_pt(*name, (*pt).into());
        }
    }
    pub fn add_pt<S: Into<String>>(&mut self, name: S, model: Point3D) {
        let name = name.into();
        let pt = Rc::new(NamedPoint::new(name.clone(), model));
        self.points.insert(name, pt);
    }
    pub fn get_pt(&self, name: &str) -> Option<Rc<NamedPoint>> {
        self.points.get(name).map(|a| a.clone())
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<String, Rc<NamedPoint>> {
        self.points.iter()
    }
}
#[test]
fn test_toml() -> Result<(), String> {
    let nps = NamedPointSet::from_toml(
        r#"
[points]
fred = {name = "fred", model = [1, 2, 3] }
"#,
    )?;
    assert!(nps.get_pt("jim").is_none(), "Jim is not a point");
    assert!(nps.get_pt("fred").is_some(), "Fred is a point");
    assert_eq!(nps.get_pt("fred").unwrap().model()[0], 1.0);
    assert_eq!(nps.get_pt("fred").unwrap().model()[1], 2.0);
    assert_eq!(nps.get_pt("fred").unwrap().model()[2], 3.0);
    Ok(())
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
