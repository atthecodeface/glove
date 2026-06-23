//tp CmdRequest
#[derive(Debug)]
pub enum CmdRequest {
    Prompt,
    Exec(String),
    ExecVec(Vec<String>),
    ExecVecSubst(Vec<String>),
    Finish,
}
impl CmdRequest {
    pub fn is_finish(&self) -> bool {
        matches!(self, CmdRequest::Finish)
    }
    pub fn is_prompt(&self) -> bool {
        matches!(self, CmdRequest::Prompt)
    }
    pub fn exec(&self) -> Option<&String> {
        if let CmdRequest::Exec(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn exec_vec(&self) -> Option<&[String]> {
        if let CmdRequest::ExecVec(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn exec_vec_subst(&self) -> Option<&[String]> {
        if let CmdRequest::ExecVecSubst(s) = self {
            Some(s)
        } else {
            None
        }
    }
}

//tp CmdResponse
#[derive(Debug)]
pub enum CmdResponse {
    Prompt(String),
    ExecOk(String),
    ExecError(String),
    Finish,
}

impl CmdResponse {
    pub fn is_finish(&self) -> bool {
        matches!(self, CmdResponse::Finish)
    }
    pub fn is_prompt(&self) -> bool {
        matches!(self, CmdResponse::Prompt(_))
    }
    pub fn is_ok(&self) -> bool {
        matches!(self, CmdResponse::ExecOk(_))
    }
    pub fn is_error(&self) -> bool {
        matches!(self, CmdResponse::ExecError(_))
    }
    pub fn take(self) -> Option<String> {
        match self {
            CmdResponse::Prompt(s) => Some(s),
            CmdResponse::ExecOk(s) => Some(s),
            CmdResponse::ExecError(s) => Some(s),
            _ => None,
        }
    }
    pub fn exec_result(&self) -> Option<&String> {
        match self {
            CmdResponse::ExecOk(s) => Some(s),
            CmdResponse::ExecError(s) => Some(s),
            _ => None,
        }
    }
    pub fn prompt(&self) -> Option<&String> {
        if let CmdResponse::Prompt(s) = self {
            Some(s)
        } else {
            None
        }
    }
}

// For flush
use std::io::Write;
pub fn shell<F, T>(mut data: T, mut request: F) -> T
where
    F: FnMut(&mut T, CmdRequest) -> CmdResponse,
{
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut has_finished = false;
    while !has_finished {
        let mut buffer = String::new();
        let response = request(&mut data, CmdRequest::Prompt);
        has_finished = response.is_finish();
        let CmdResponse::Prompt(prompt) = response else {
            break;
        };
        print!("{prompt}");

        if let Err(e) = stdout.flush() {
            println!("Error: {e}");
            break;
        }

        if stdin.read_line(&mut buffer).is_err() {
            break;
        }
        if buffer.is_empty() {
            break;
        }

        let response = request(&mut data, CmdRequest::Exec(buffer));
        has_finished = response.is_finish();
        match response {
            CmdResponse::ExecOk(_s) => {}
            CmdResponse::ExecError(e) => {
                println!("Error: {e}");
            }
            _ => {
                break;
            }
        }
    }
    // eprintln!("Has finished? {has_finished}");
    if !has_finished {
        let _ = request(&mut data, CmdRequest::Finish);
    }
    data
}
