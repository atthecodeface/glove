//a Imports
use std::path::{Component, PathBuf};

//a UriDecode
//tp UriDecode
#[derive(Debug, Default)]
pub struct UriDecode {
    uri: Option<String>,
    path: Option<PathBuf>,
    action: Option<String>,
    args: Vec<(String, Option<String>)>,
}

//ip UriDecode
impl UriDecode {
    //ap uri
    pub fn uri(&self) -> Option<&String> {
        self.uri.as_ref()
    }

    //ap path
    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    //ap action
    pub fn action(&self) -> Option<&String> {
        self.action.as_ref()
    }

    //ap args
    pub fn args(&self) -> &[(String, Option<String>)] {
        &self.args
    }

    //cp of_uri
    fn of_uri(uri: &str) -> Self {
        let uri = uri.to_string();
        Self {
            uri: Some(uri),
            path: None,
            action: None,
            args: vec![],
        }
    }

    //cp of_path
    fn of_path(path: PathBuf) -> Self {
        Self {
            uri: None,
            path: Some(path),
            action: None,
            args: vec![],
        }
    }

    //mp set_action
    fn set_action(&mut self, action: Option<&str>) {
        self.action = action.map(|a| a.to_lowercase());
    }

    //mp add_arg
    fn add_arg(&mut self, arg: &str, value: Option<&str>) {
        self.args
            .push((arg.to_lowercase(), value.map(|a| a.to_owned())));
    }

    //fp canonicalize_path
    pub fn canonicalize_path(path: &str) -> Option<PathBuf> {
        let mut pb = PathBuf::new();
        for pc in PathBuf::from(path).components() {
            match pc {
                Component::RootDir => {
                    pb = PathBuf::new();
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    if !pb.pop() {
                        return None;
                    }
                }
                Component::Normal(pc) => {
                    pb.push(pc);
                }
                _ => {
                    // C: for example on Windows
                    return None;
                }
            }
        }
        Some(pb)
    }

    //cp decode_uri
    /// Parse a URI as a path optionally followed by ? action [& k=v]*
    ///
    /// If the decode fails, produce a plain Uri
    pub fn decode_uri(uri: &str) -> UriDecode {
        let mut split = uri.splitn(2, '?');
        let Some(uri) = Self::canonicalize_path(split.next().unwrap()) else {
            return UriDecode::of_uri(uri);
        };

        let mut ud = UriDecode::of_path(uri);
        if let Some(action_args) = split.next() {
            let mut aa_split = action_args.split('&');
            ud.set_action(aa_split.next());
            for args in aa_split {
                let mut arg_split = args.splitn(2, '=');
                let arg = arg_split.next().unwrap();
                ud.add_arg(arg, arg_split.next());
            }
        }
        ud
    }

    //ap action_is
    pub fn action_is(&self, action: &str) -> bool {
        self.action.as_ref().is_some_and(|a| a == action)
    }

    //ap try_get_one
    // #[inline]
    // fn try_get_arg(&self, arg: &str) -> Result<Option<&MatchedArg>, MatchesError> {
    // Ok(self.args.get(arg))
    // }

    // #[inline]
    // fn try_get_arg_t<T: Any + 'static>(
    // &self,
    // arg: &str,
    // ) -> Result<Vec<UiArgValue>, String> {
    // let arg = match self.try_get_arg(arg) {
    // Some(arg) => arg,
    // None => {
    // return Ok(None);
    // }
    // };
    // ok!(self.verify_arg_t::<T>(arg));
    // Ok(Some(arg))
    // }

    //zz All done
}
