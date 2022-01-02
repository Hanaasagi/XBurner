use std::process::{exit, Command, Stdio};

use nix::sys;
use nix::unistd;

// The parent forks the child
// The parent returns
// The child calls setsid() to start a new session with no controlling terminals
// The child forks a grandchild
// The child exits
// The grandchild is now the daemon
pub fn execute(shell: &str)
//where
//    S: AsRef<OsStr>,
//    I: IntoIterator<Item = S>,
{
    unsafe {
        match unistd::fork().expect("Failed to fork process") {
            unistd::ForkResult::Parent { child } => {
                // wait child exited
                sys::wait::waitpid(Some(child), None).unwrap();
            }

            unistd::ForkResult::Child => {
                // make child new session leader
                unistd::setsid().expect("Failed to set sid");
                // change child work dir
                // unistd::chdir("/").expect("Failed to change dir");

                //nix::unistd::close(0);
                //nix::unistd::close(1);
                //nix::unistd::close(2);
                // Close all open file descripter

                for fd in 0..unistd::sysconf(unistd::SysconfVar::OPEN_MAX)
                    .map(|m| m.unwrap_or(0))
                    .unwrap()
                {
                    unistd::close(fd as i32).ok();
                }

                // TODO: maybe a log file
                Command::new("sh")
                    .arg("-c")
                    .arg(shell)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("failed to spawn the target process");

                // The child exits
                exit(0);
            }
        }
    }
}
