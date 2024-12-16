use std::process::Output;

#[derive(Clone)]
pub struct CmdOutput {
    pub status: i32,
    pub stdout: Option<Vec<u8>>,
    pub stderr: Option<Vec<u8>>
}

impl CmdOutput {
    pub fn new() -> CmdOutput {
        return CmdOutput{
            status: 0,
            stdout: Some(Vec::new()),
            stderr: None
        }
    }
    pub fn from_output(out: &Output) -> CmdOutput {
        let mut code = 0;
        if let Some(exitstatus) = out.status.code() {
            code = exitstatus;
        }
        return CmdOutput{
            status: code,
            stdout: Some(out.stdout.clone()),
            stderr: Some(out.stderr.clone())
        }
    }

    pub fn from_status(exitcode: i32) -> CmdOutput {
        return CmdOutput{
            status: exitcode,
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