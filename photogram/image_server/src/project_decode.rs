//a Imports
use ic_http::HttpRequest;

//a ProjectDecode
//tp ProjectDecodeType
#[derive(Debug, Default)]
pub enum ProjectDecodeType {
    #[default]
    Root,
    UnknownProject,
    Project,
}

//tp ProjectDecode
#[derive(Debug, Default)]
pub struct ProjectDecode {
    pub dec_type: ProjectDecodeType,
    pub project: String,
    pub idx: usize,
    pub cip: Option<usize>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub px_per_model: Option<f64>,
    pub window: Option<usize>,
    pub radius: Option<usize>,
    pub nps: Vec<String>,
}

//ip ProjectDecode
impl ProjectDecode {
    //cp decode_request
    pub fn decode_request(request: &HttpRequest) -> Option<Self> {
        let path = request.uri.path()?;
        if !path.starts_with("project") {
            return None;
        }
        let project = path.strip_prefix("project").unwrap();
        let project = project.to_str()?;
        let mut pd = ProjectDecode {
            dec_type: ProjectDecodeType::Root,
            ..Default::default()
        };
        if !project.is_empty() {
            pd.dec_type = ProjectDecodeType::UnknownProject;
            pd.project = project.into();
        }
        if let Some(Ok(cip)) = request.get_one::<usize>("cip") {
            pd.cip = Some(cip);
        }
        if let Some(Ok(width)) = request.get_one::<f64>("width") {
            pd.width = Some(width);
        }
        if let Some(Ok(height)) = request.get_one::<f64>("height") {
            pd.height = Some(height);
        }
        if let Some(Ok(px_per_model)) = request.get_one::<f64>("px_per_model") {
            pd.px_per_model = Some(px_per_model);
        }
        if let Some(Ok(window)) = request.get_one::<usize>("window") {
            pd.window = Some(window);
        }
        if let Some(Ok(radius)) = request.get_one::<usize>("window") {
            pd.radius = Some(radius);
        }
        for np in request.get_many::<String>("np").flatten() {
            pd.nps.push(np);
        }
        Some(pd)
    }

    //mp set_project_idx
    pub fn set_project_idx(&mut self, idx: Option<usize>) {
        if let Some(idx) = idx {
            self.dec_type = ProjectDecodeType::Project;
            self.idx = idx;
        } else {
            self.dec_type = ProjectDecodeType::UnknownProject;
            self.idx = 0;
        }
    }

    //ap is_root
    pub fn is_root(&self) -> bool {
        matches!(self.dec_type, ProjectDecodeType::Root)
    }

    //ap is_project
    fn is_project(&self) -> bool {
        matches!(self.dec_type, ProjectDecodeType::Project)
    }

    //ap might_be_project
    pub fn might_be_project(&self) -> bool {
        matches!(
            self.dec_type,
            ProjectDecodeType::Project | ProjectDecodeType::UnknownProject
        )
    }

    //ap project_idx
    pub fn project_idx(&self) -> Option<usize> {
        self.is_project().then_some(self.idx)
    }

    //ap project
    pub fn project(&self) -> Option<&str> {
        match self.dec_type {
            ProjectDecodeType::Project => Some(&self.project),
            ProjectDecodeType::UnknownProject => Some(&self.project),
            _ => None,
        }
    }
    //ap cip
    pub fn cip(&self) -> Option<usize> {
        self.cip
    }

    //zz All done
}
