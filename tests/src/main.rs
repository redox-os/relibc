#![feature(exit_status_error)]

use std::{
    env, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{self, Command, ExitStatus, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

fn find_expected_dir() -> Result<PathBuf, String> {
    let mut current_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|parent| parent.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let mut found_expected_dir = None;

    while current_dir.pop() {
        let check = current_dir.join("expected");
        if check.is_dir() {
            found_expected_dir = Some(check);
            break;
        }
    }

    found_expected_dir.ok_or_else(|| "Could not find 'expected' directory".to_string())
}

fn expected(
    expected_dir: &Path,
    bin: &str,
    kind: &str,
    generated: &[u8],
    status: ExitStatus,
) -> Result<(), String> {
    let expect_file = Path::new(bin).with_added_extension(kind);
    let components: Vec<_> = expect_file
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .rev()
        .collect();

    let mut expected_file = None;
    for i in 0..components.len() {
        let sub_path: Vec<_> = components[0..=i]
            .iter()
            .rev()
            .map(|s| s.to_string())
            .collect();

        let check_file = expected_dir.join(sub_path.join("/"));
        if check_file.is_file() {
            expected_file = Some(fs::read(check_file));
            break;
        }
    }

    let expect_name = components.first().unwrap();
    let expected = match expected_file {
        Some(Ok(ok)) => ok,
        Some(Err(err)) => {
            return Err(format!("{} failed to read {}: {}", bin, expect_name, err));
        }
        None => {
            if kind == "stderr" {
                // missing stderr file, assume test expect none emitted
                vec![]
            } else {
                return Err(format!("{} expected file not found: {}", bin, expect_name));
            }
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

fn print_tabbed(output: Vec<u8>, name: &str) {
    if let Ok(stdout) = String::from_utf8(output) {
        let stdout: Vec<String> = stdout.trim().lines().map(|p| format!("  {}", p)).collect();
        let stdout = stdout.join("\n");
        if stdout.as_str() == "" {
            println!("{name}: empty content");
        } else {
            println!("{name}:\n{}", stdout);
        }
    } else {
        println!("can't print out {name}: not utf8");
    }
}

fn main() {
    let mut failures = Vec::new();
    let timeout = Duration::from_secs(10);
    let slowtime = Duration::from_secs(1);
    let bins: Vec<String> = env::args().skip(1).collect();
    let single_test = bins.len() == 1;
    let expected_dir = find_expected_dir();

    for bin in bins {
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

            if start_time.elapsed() > slowtime {
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
            (exit_status, Some(mut child)) => {
                if exit_status.is_none() {
                    match child.kill() {
                        Ok(_) => {}
                        Err(e) => {
                            // if can't be killed then getting the output will hang
                            println!("Unable to kill, can't get output: {:?}", e);
                            continue;
                        }
                    }
                }
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                child.stdout.unwrap().read_to_end(&mut stdout).unwrap();
                child.stderr.unwrap().read_to_end(&mut stderr).unwrap();

                let Some(exit_status) = exit_status else {
                    // hangs
                    print_tabbed(stdout, "stdout");
                    print_tabbed(stderr, "stderr");
                    continue;
                };

                if !status_only {
                    let Ok(expected_dir) = &expected_dir else {
                        eprintln!("Expected directory not found");
                        process::exit(1);
                    };

                    if let Err(failure) =
                        expected(expected_dir, &bin, "stdout", &stdout, exit_status)
                    {
                        println!("{}", failure);
                        failures.push(failure);
                    }
                    if let Err(failure) =
                        expected(expected_dir, &bin, "stderr", &stderr, exit_status)
                    {
                        println!("{}", failure);
                        failures.push(failure);
                    }
                }

                if let Err(e) = exit_status.exit_ok() {
                    let failure = format!("# {}: {}", bin, e);
                    println!("{}", failure);
                    failures.push(failure);
                }

                if single_test {
                    print_tabbed(stdout, "stdout");
                    print_tabbed(stderr, "stderr");
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
