use crossterm::style::Print;
use crossterm::QueueableCommand;
use crate::core::core::ShellState;

pub fn poll_jobs(state: &mut ShellState) {
    let mut i = 0;
    while i < state.jobs.len() {
        let (job_id, name, mut child) = state.jobs.remove(i);
        match child.try_wait() {
            Ok(Some(_status)) => {
                // For jobs with inherited I/O, wait() is enough; we keep this branch for state reporting.
                let _ = child.wait();
                #[cfg(unix)] {
                    use std::os::unix::process::ExitStatusExt;
                    if status.signal() == Some(19) { // SIGTSTP
                        state.stdout.queue(Print(format!("[{}]+ Stopped {}\n", job_id, name))).unwrap();
                        state.jobs.insert(i, (job_id, name, child));
                        i += 1;
                        continue;
                    }
                }
                state.stdout.queue(Print(format!("[{}]+ Done {}\n", job_id, name))).unwrap();
                // Job completed, don't insert back
            },
            Ok(None) => {
                state.jobs.insert(i, (job_id, name, child));
                i += 1;
            },
            Err(_) => {
                // Error, don't insert back
            }
        }
    }
    state.stdout.flush().unwrap();
}