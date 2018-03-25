use std::env;
use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus};

fn main() {
    make("openlibm").expect("failed to build openlibm");
}

/// Changes the current directory to the specified path, executes the `make`
/// command, and then changes the current directory back to the original path.
fn make<P: AsRef<Path>>(path: P) -> io::Result<ExitStatus> {
    env::current_dir().and_then(|pwd| {
        env::set_current_dir(path)
            .and(Command::new("make").status())
            .and_then(|status| env::set_current_dir(pwd).and(Ok(status)))
    })
}
