use geo_nd::Vector;
use thunderclap::{CmdDescriptor, json};

use ic_photogram::CameraProjection;

use crate::cmd::{CmdArgs, CmdResult};

impl CmdArgs {
    fn project_list_cips_cmd(&mut self) -> CmdResult {
        let ncips = self.project().ncips();
        let mut result = vec![];
        for i in 0..ncips {
            let name = self.project().cip_name(i).unwrap();
            if self.verbose {
                let cip = self.project().find_cip(&name).unwrap().clone();
                let cip = cip.borrow();
                let camera = cip.camera();
                let position = camera.borrow().position().clone();
                let is_placed = !position.is_zero();
                if is_placed {
                    println!("Cip: '{name}' @ {position}",);
                } else {
                    println!("Cip: '{name}' camera unplaced");
                }
            }
            result.push(name);
        }
        Ok(json::to_value(result)?)
    }

    fn project_as_json_cmd(&mut self) -> CmdResult {
        if let Some(w) = self.write_project.as_ref() {
            let s = self.project().to_json(true)?;
            std::fs::write(w, s)?;
        }
        Ok(json::to_value(self.project().to_json(self.pretty_json())?)?)
    }

    const PROJECT_CIPS_CMD: CmdDescriptor<Self> = CmdDescriptor::new("cips")
        .about("Get a list of the names of the CIPs in the project; if verbose, display their placement")
        .args(&[])
        .handler(&Self::project_list_cips_cmd);

    const PROJECT_AS_JSON_CMD: CmdDescriptor<Self> = CmdDescriptor::new("as_json")
        .about("Generate the JSON for a project; if the 'write project' option is set, then it is written to that file (prettily)")
        .args(&[])
        .handler(&Self::project_as_json_cmd);

    pub(crate) const PROJECT_CMD: CmdDescriptor<Self> = CmdDescriptor::new("project")
        .about("Operate on the project as a whole")
        .args(&[])
        .cmds(&[Self::PROJECT_CIPS_CMD, Self::PROJECT_AS_JSON_CMD]);
}
