use std::process::{Output, ExitStatus};

pub struct CmdOutput {
    pub status: ExitStatus,
    pub stdout: Option<Vec<u8>>,
    pub stderr: Option<Vec<u8>>
}

impl CmdOutput {
    pub fn from_output(out: &Output) -> CmdOutput {
        return CmdOutput{
            status: out.status,
            stdout: Some(out.stdout.clone()),
            stderr: Some(out.stderr.clone())
        }
    }

    pub fn from_status(code: ExitStatus) -> CmdOutput {
        return CmdOutput{
            status: code,
            stdout: None,
            stderr: None
        }
    }
}