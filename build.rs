extern crate cc;

use std::{env, fs, process::Command};

fn try_get_commit_hash(crate_dir: &str) -> String {
    eprintln!("relibc crate dir: `{crate_dir}`");
    let child = match Command::new("git")
        .arg("-C")
        .arg(crate_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("relibc build.rs: failed to run git command to get hash: {e}");
            return "(unknown; git command failed)".into();
        }
    };
    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("relibc build.rs: failed to wait and get git's output: {e}");
            return "(unknown; git output failed)".into();
        }
    };
    if !output.status.success() {
        eprintln!("git stderr: `{}`", String::from_utf8_lossy(&output.stderr));
        return "(unknown; git command failed)".into();
    }

    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let commit_hash = try_get_commit_hash(&crate_dir);
    eprintln!("relibc: commit hash `{commit_hash}`");
    println!("cargo:rustc-env=RELIBC_COMMIT_HASH={commit_hash}");

    let target = env::var("TARGET").unwrap();

    println!("cargo:rerun-if-changed=src/c");

    let mut cc_builder = &mut cc::Build::new();

    cc_builder = cc_builder.flag("-nostdinc").flag("-nostdlib");

    if target.starts_with("aarch64") {
        cc_builder = cc_builder.flag("-mno-outline-atomics")
    }

    cc_builder
        .flag("-fno-stack-protector")
        .flag("-Wno-expansion-to-defined")
        .files(
            fs::read_dir("src/c")
                .expect("src/c directory missing")
                .map(|res| res.expect("read_dir error").path()),
        )
        .compile("relibc_c");

    println!("cargo:rustc-link-lib=static=relibc_c");
}
