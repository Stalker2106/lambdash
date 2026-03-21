use crossterm::style::Print;
use crossterm::QueueableCommand;
use crate::core::core::ShellState;

pub fn poll_jobs(state: &mut ShellState) {
    let mut i = 0;
    while i < state.jobs.len() {
        let (job_id, name, child) = &mut state.jobs[i];
        match child.try_wait() {
            Ok(Some(status)) => {
                #[cfg(unix)] {
                    use std::os::unix::process::ExitStatusExt;
                    if status.signal() == Some(19) { // SIGTSTP
                        state.stdout.queue(Print(format!("[{}]+ Stopped {}\n", job_id, name))).unwrap();
                        i += 1;
                        continue;
                    }
                }
                state.stdout.queue(Print(format!("[{}] Done {}\n", job_id, name))).unwrap();
                state.jobs.remove(i);
            },
            Ok(None) => { i += 1; },
            Err(_) => { state.jobs.remove(i); }
        }
    }
    state.stdout.flush().unwrap();
}