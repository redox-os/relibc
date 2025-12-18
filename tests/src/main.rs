use std::{
    env, fs,
    io::{self, Read, Write},
    path::PathBuf,
    process::{self, Command, ExitStatus, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

fn expected(bin: &str, kind: &str, generated: &[u8], status: ExitStatus) -> Result<(), String> {
    let mut expected_file = PathBuf::from(format!("expected/{}.{}", bin, kind));
    if !expected_file.exists() {
        expected_file = PathBuf::from(format!(
            "expected/{}.{}",
            bin.replace("bins_static", "").replace("bins_dynamic", ""),
            kind
        ));
    }

    let expected = match fs::read(&expected_file) {
        Ok(ok) => ok,
        Err(err) => {
            return Err(format!(
                "{} failed to read {}: {}",
                bin,
                expected_file.display(),
                err
            ));
        }
    };

    if expected != generated {
        println!("# {}: {}: expected #", bin, kind);
        io::stdout().write(&expected).unwrap();

        println!("# {}: {}: generated #", bin, kind);
        io::stdout().write(generated).unwrap();

        return Err(format!(
            "{} failed - retcode {}, {} mismatch",
            bin, status, kind
        ));
    }

    Ok(())
}

const STATUS_ONLY: &str = "-s";

fn main() {
    let mut failures = Vec::new();
    let timeout = Duration::from_secs(10);
    let slowtime = Duration::from_secs(1);

    for bin in env::args().skip(1) {
        let status_only = bin.starts_with(STATUS_ONLY);
        let bin = if bin.starts_with(STATUS_ONLY) {
            bin.strip_prefix(STATUS_ONLY).unwrap().to_string()
        } else {
            bin
        };

        println!("# {} #", bin);

        let start_time = Instant::now();

        let (tx, rx) = mpsc::channel();
        let bin_for_spawn = bin.clone();

        // There's an issue when pthread hangs, spawn() also hangs, so let's use separate thread to spawn
        thread::spawn(move || {
            let result = Command::new(&bin_for_spawn)
                .arg("test")
                .arg("args")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();
            tx.send(result).expect("Can't send");
        });

        let mut status = None;
        let mut child = None;

        loop {
            if start_time.elapsed() > timeout {
                let failure = format!(
                    "\x1b[0;91;49m{}: assumed hangs after {}ms\x1b[0m",
                    bin,
                    start_time.elapsed().as_millis()
                );
                println!("{}", failure);
                failures.push(failure);
                break;
            }
            // A pretty rare hang on pthread/barrier which also stops this loop :(
            // https://gitlab.redox-os.org/redox-os/relibc/-/issues/238
            if &bin == "./bins_dynamic/pthread/barrier" || start_time.elapsed() > slowtime {
                println!("# waiting {}ms", start_time.elapsed().as_millis());
            }

            let c = match &mut child {
                Some(child) => child,
                None => match rx.try_recv() {
                    Ok(Ok(c)) => {
                        child = Some(c);
                        continue;
                    }
                    Ok(Err(err)) => {
                        let failure = format!("{}: failed to execute: {}", bin, err);
                        println!("{}", failure);
                        failures.push(failure);
                        break;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        let failure = format!("{}: failed to execute: thread died", bin);
                        println!("{}", failure);
                        failures.push(failure);
                        break;
                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        thread::sleep(Duration::from_millis(if start_time.elapsed() < slowtime {
                            25
                        } else {
                            500
                        }));
                        continue;
                    }
                },
            };

            match c.try_wait() {
                Ok(Some(s)) => {
                    status = Some(s);
                    break;
                }
                Ok(None) => {
                    thread::sleep(Duration::from_millis(if start_time.elapsed() < slowtime {
                        25
                    } else {
                        500
                    }));
                }
                Err(e) => {
                    failures.push(format!("{}: error waiting: {}", bin, e));
                    break;
                }
            }
        }

        match (status, child) {
            (Some(exit_status), Some(child)) => {
                if !status_only {
                    let mut stdout = Vec::new();
                    let mut stderr = Vec::new();
                    child.stdout.unwrap().read_to_end(&mut stdout).unwrap();
                    child.stderr.unwrap().read_to_end(&mut stderr).unwrap();

                    if let Err(failure) = expected(&bin, "stdout", &stdout, exit_status) {
                        println!("{}", failure);
                        failures.push(failure);
                    }
                    if let Err(failure) = expected(&bin, "stderr", &stderr, exit_status) {
                        println!("{}", failure);
                        failures.push(failure);
                    }
                }
            }
            (_, _) => {
                continue;
            }
        }

        if start_time.elapsed() > slowtime {
            println!(
                "\x1b[0;93;49m  test exection took too long: {}ms\x1b[0m",
                start_time.elapsed().as_millis()
            );
        }
    }

    if !failures.is_empty() {
        println!("\x1b[1;91;49m# FAILURES #\x1b[0m");
        for failure in failures {
            println!("{}", failure);
        }
        process::exit(1);
    }
}
