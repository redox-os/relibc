use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
    process::{self, Command, ExitStatus},
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

fn main() {
    let mut failures = Vec::new();

    for bin in env::args().skip(1) {
        println!("# {} #", bin);

        match Command::new(&bin).arg("test").arg("args").output() {
            Ok(output) => {
                if let Err(failure) = expected(&bin, "stdout", &output.stdout, output.status) {
                    println!("{}", failure);
                    failures.push(failure);
                }

                if let Err(failure) = expected(&bin, "stderr", &output.stderr, output.status) {
                    println!("{}", failure);
                    failures.push(failure);
                }
            }
            Err(err) => {
                let failure = format!("{}: failed to execute: {}", bin, err);
                println!("{}", failure);
                failures.push(failure);
            }
        }
    }

    if !failures.is_empty() {
        println!("# FAILURES #");
        for failure in failures {
            println!("{}", failure);
        }
        process::exit(1);
    }
}
