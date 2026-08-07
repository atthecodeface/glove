use std::marker::PhantomData;
use std::path::Path;

use geo_nd::Quaternion;

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{Error, PathSet, Point2D, Point3D, Quat, Result};

/// A type parsable by Json which takes a descriptor and, post parsing, can generate another object
pub trait JsonParsable: serde::de::DeserializeOwned {
    type PostParseArg;
    type PostParseResult: Sized;
    fn reason() -> &'static str;
    fn post_parse(self, _args: &Self::PostParseArg) -> Result<Self::PostParseResult>;
    fn load_json_file<P: AsRef<Path> + std::fmt::Display>(
        path_set: &PathSet,
        path: P,
        post_parse: &Self::PostParseArg,
    ) -> Result<(String, Self::PostParseResult)> {
        JsonSrc::<Self>::load_json_file(path_set, path, post_parse)
    }
    fn load_json<A: AsRef<str>>(
        json: A,
        post_parse: &Self::PostParseArg,
    ) -> Result<Self::PostParseResult> {
        JsonSrc::<Self>::load_json(json, post_parse)
    }
}

macro_rules! json_parsable {
{ $t:ty, $reason:expr } => {
    impl JsonParsable for $t {
    type PostParseArg = ();
    type PostParseResult = $t;
    fn reason() -> &'static str {
        $reason
    }
    fn post_parse(self, _args: &()) -> Result<Self> {
        Ok(self)
    }
}
}
}
json_parsable!((), "general");
json_parsable!(f32, "f32");
json_parsable!(f64, "f64");
json_parsable!(bool, "bool");
json_parsable!(usize, "usize");

impl<T: JsonParsable> JsonParsable for Vec<T> {
    type PostParseArg = T::PostParseArg;
    type PostParseResult = Vec<T::PostParseResult>;
    fn reason() -> &'static str {
        "vec<>"
    }
    fn post_parse(self, args: &Self::PostParseArg) -> Result<Vec<T::PostParseResult>> {
        let mut result = vec![];
        for s in self.into_iter() {
            result.push(s.post_parse(args)?);
        }
        Ok(result)
    }
}

//a JsonSrc
//tp JsonSrc
pub struct JsonSrc<T: JsonParsable> {
    filename: String,
    reason: &'static str,
    json_string: String,
    phantom: PhantomData<T>,
}

impl<T> JsonSrc<T>
where
    T: JsonParsable,
{
    //fp remove_comments
    fn remove_comments<A: AsRef<str>>(s: A) -> String {
        let mut json_string = String::new();
        for j in s.as_ref().lines() {
            if let Some(n) = j.find("//") {
                json_string.push_str(j.split_at(n).0);
                json_string.push('\n');
            } else {
                json_string.push_str(j);
                json_string.push('\n');
            }
        }
        json_string
    }

    //cp of_json
    pub fn of_json<A: AsRef<str>>(json: A) -> Result<Self> {
        Ok(Self {
            filename: "".into(),
            reason: T::reason(),
            json_string: Self::remove_comments(json),
            phantom: PhantomData,
        })
    }

    //cp read_json_file
    pub fn read_json_file<P: AsRef<Path> + std::fmt::Display>(
        path_set: &PathSet,
        path: P,
    ) -> Result<Self> {
        let path = path_set.find_file_err(&path)?;
        let filename = path.display();
        let file_text = std::fs::read_to_string(&path)?;
        Ok(Self {
            filename: filename.to_string(),
            reason: T::reason(),
            json_string: Self::remove_comments(file_text),
            phantom: PhantomData,
        })
    }

    //fi json_error
    pub fn json_error(&self, err: serde_json::Error) -> Error {
        let line = err.line();
        let column = err.column();
        let mut result = format!(
            "Error in parsing '{}' as {} json at line {line} column {column}",
            self.filename, self.reason,
        );
        let start_line = line.saturating_sub(5);
        let end_line = line + 5;
        for (i, l) in self.json_string.lines().enumerate().skip(start_line) {
            if i >= start_line {
                result.push_str(&format!("\n{:4}: {}", i + 1, l));
            }
            if i >= end_line {
                break;
            }
        }
        Error::JsonCtxt(result, err)
    }

    //fp deserialize_as
    pub fn deserialize_as<D: DeserializeOwned>(
        mut self,
        reason: &'static str,
    ) -> Result<(String, D)> {
        self.reason = reason;
        let d = serde_json::from_str(&self.json_string).map_err(|e| self.json_error(e))?;
        Ok((self.filename, d))
    }

    //fp deserialize
    pub fn deserialize(self) -> Result<(String, T)> {
        let t = serde_json::from_str(&self.json_string).map_err(|e| self.json_error(e))?;
        Ok((self.filename, t))
    }

    //fp load_json
    pub fn load_json<A: AsRef<str>>(
        json: A,
        post_parse: &T::PostParseArg,
    ) -> Result<T::PostParseResult> {
        let json = Self::of_json(json)?;
        json.deserialize()?.1.post_parse(post_parse)
    }

    //fp load_json_file
    pub fn load_json_file<P: AsRef<Path> + std::fmt::Display>(
        path_set: &PathSet,
        path: P,
        post_parse: &T::PostParseArg,
    ) -> Result<(String, T::PostParseResult)> {
        let json = Self::read_json_file(path_set, path)?;
        let (filename, result) = json.deserialize()?;
        Ok((filename, result.post_parse(post_parse)?))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum QuaternionDesc {
    Quaternion { i: f64, j: f64, k: f64, r: f64 },
    LookAt { at: [f64; 3], up: [f64; 3] },
}
impl JsonParsable for QuaternionDesc {
    type PostParseArg = ();
    type PostParseResult = Quat;
    fn reason() -> &'static str {
        "orientation"
    }
    fn post_parse(self, _args: &()) -> Result<Quat> {
        let q = match self {
            QuaternionDesc::Quaternion { r, i, j, k } => Quat::of_rijk(r, i, j, k),
            QuaternionDesc::LookAt { at, up } => Quat::look_at(&at.into(), &up.into()),
        };
        Ok(q.normalize())
    }
}

impl JsonParsable for Point2D {
    type PostParseArg = ();
    type PostParseResult = Point2D;
    fn reason() -> &'static str {
        "2d point"
    }
    fn post_parse(self, _args: &()) -> Result<Point2D> {
        Ok(self)
    }
}

impl JsonParsable for Point3D {
    type PostParseArg = ();
    type PostParseResult = Point3D;
    fn reason() -> &'static str {
        "2d point"
    }
    fn post_parse(self, _args: &()) -> Result<Point3D> {
        Ok(self)
    }
}
