use std::process::Output;

#[derive(Clone)]
pub struct CmdOutput {
    pub status: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>
}

impl CmdOutput {
    pub fn new() -> CmdOutput {
        return CmdOutput{
            status: None,
            stdout: Vec::new(),
            stderr: Vec::new()
        }
    }
    pub fn from_output(out: &Output, capture_output: bool) -> CmdOutput {
        let mut code = 0;
        if let Some(exitstatus) = out.status.code() {
            code = exitstatus;
        }
        return CmdOutput{
            status: Some(code),
            stdout: if capture_output { out.stdout.clone() } else { Vec::new() },
            stderr: if capture_output { out.stderr.clone() } else { Vec::new() }
        };
    }

    pub fn from_status(exitcode: i32) -> CmdOutput {
        return CmdOutput{
            status: Some(exitcode),
            stdout: Vec::new(),
            stderr: Vec::new()
        }
    }
}