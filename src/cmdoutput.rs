use std::process::{Output, ExitStatus};
use std::os::unix::process::ExitStatusExt;

#[derive(Clone)]
pub struct CmdOutput {
    pub status: ExitStatus,
    pub stdout: Option<Vec<u8>>,
    pub stderr: Option<Vec<u8>>
}

impl CmdOutput {
    pub fn new() -> CmdOutput {
        return CmdOutput{
            status: ExitStatus::from_raw(0),
            stdout: Some(Vec::new()),
            stderr: None
        }
    }
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

    pub fn combine(&mut self, out: &CmdOutput) {
        if let Some(e_stdout) = out.stdout.clone() {
            if let Some(stdout) = &mut self.stdout {
                stdout.extend(e_stdout);
            } else {
                self.stdout = Some(e_stdout);
            }
        }
    }
}