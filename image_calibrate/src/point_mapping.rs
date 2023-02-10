//a Imports
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{json, NamedPoint, NamedPointSet, Point2D, Point3D};

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
    /// Error in pixels
    pub error: f64,
}

//ip Serialize for PointMapping
impl Serialize for PointMapping {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeTuple;
        let mut seq = serializer.serialize_tuple(3)?;
        seq.serialize_element(self.model.name())?;
        seq.serialize_element(&self.screen)?;
        seq.serialize_element(&self.error)?;
        seq.end()
    }
}

//ip Deserialize for PointMapping
impl<'de> Deserialize<'de> for PointMapping {
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let (model_name, screen, error) = <(String, Point2D, f64)>::deserialize(deserializer)?;
        let model = Rc::new(NamedPoint::new(model_name, [0., 0., 0.].into()));
        Ok(Self {
            model,
            screen,
            error,
        })
    }
}

//ip PointMapping
impl PointMapping {
    //fp new_npt
    pub fn new_npt(model: Rc<NamedPoint>, screen: &Point2D, error: f64) -> Self {
        PointMapping {
            model,
            screen: *screen,
            error,
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

    //mp model
    #[inline]
    pub fn model(&self) -> Point3D {
        self.model.model()
    }

    //mp screen
    #[inline]
    pub fn screen(&self) -> Point2D {
        self.screen
    }

    //mp error
    #[inline]
    pub fn error(&self) -> f64 {
        self.error
    }

    //mp name
    pub fn name(&self) -> &str {
        self.model.name()
    }

    //zz All done
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
    pub fn from_json(nps: &NamedPointSet, json: &str) -> Result<Self, String> {
        let mut pms: Self = json::from_json("point map set", json)?;
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
            self.add_mapping(nps, name, &[*px as f64, *py as f64].into(), 5.0);
        }
    }

    //mp add_mapping
    pub fn add_mapping(
        &mut self,
        nps: &NamedPointSet,
        name: &str,
        screen: &Point2D,
        error: f64,
    ) -> bool {
        if let Some(model) = nps.get_pt(name) {
            self.mappings
                .push(PointMapping::new_npt(model, screen, error));
            true
        } else {
            false
        }
    }

    //ap mappings
    pub fn mappings(&self) -> &[PointMapping] {
        &self.mappings
    }

    //ap mapping_of_np
    pub fn mapping_of_np(&self, np: &Rc<NamedPoint>) -> Option<&PointMapping> {
        for pm in self.mappings.iter() {
            if Rc::ptr_eq(np, &pm.model) {
                return Some(pm);
            }
        }
        None
    }
}

//a Tests
//ft test_json_1
#[test]
fn test_json_1() -> Result<(), String> {
    let mut nps = NamedPointSet::default();
    nps.add_pt("fred", [1., 2., 3.].into());
    let mut pms = PointMappingSet::new();
    pms.add_mapping(&nps, "fred", &[1., 2.].into(), 5.0);
    let s = pms.to_json()?;
    assert_eq!(s, r#"[["fred",[1.0,2.0],5.0]]"#);
    let pms = PointMappingSet::from_json(
        &nps,
        r#"
[["fred", [1, 2], 5.0]]
"#,
    )?;
    assert_eq!(pms.mappings().len(), 1);
    assert_eq!(pms.mappings()[0].name(), "fred");
    assert_eq!(pms.mappings()[0].screen()[0], 1.0);
    assert_eq!(pms.mappings()[0].screen()[1], 2.0);
    Ok(())
}
