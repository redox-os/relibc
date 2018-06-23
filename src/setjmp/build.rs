extern crate cbindgen;

use std::{env, fs, process::Command};

fn compile(file: &str, object: &str, output: &str) {
    let status = Command::new("gcc")
        .args(&["-c", file, "-o", object])
        .status()
        .expect("failed to run gcc to compile assembly");

    if !status.success() {
        panic!("compilation error");
    }

    let status = Command::new("ar")
        .args(&["rcs", output, object])
        .status()
        .expect("failed to run ar to convert object to a static library");

    if !status.success() {
        panic!("error converting object to static library");
    }
}

fn main() {
    println!("cargo:rustc-link-lib=static=setjmp");
    println!("cargo:rustc-link-lib=static=longjmp");

    macro_rules! detect_arch {
        ($($($token:tt);+),+) => {
            $(
                detect_arch!(inner $($token);+);
            )+
        };
        (inner $arch:expr) => {
            detect_arch!(inner $arch; ".s");
        };
        (inner $arch:expr; $ext:expr) => {
            #[cfg(target_arch = $arch)] {
                compile(concat!("impl/", $arch, "/setjmp", $ext), "impl/bin/setjmp.o", "impl/bin/libsetjmp.a");
                compile(concat!("impl/", $arch, "/longjmp", $ext), "impl/bin/longjmp.o", "impl/bin/liblongjmp.a");

                let dir = env::current_dir().expect("failed to find current directory");
                println!("cargo:rustc-link-search=native={}/impl/bin", dir.display());
            }
        };
    }

    detect_arch! {
        "aarch64",
        "arm",
        "i386",
        "m68k",
        "microblaze",
        "mips"; ".S",
        "mips64"; ".S",
        "mipsn32"; ".S",
        "or1k",
        "powerpc"; ".S",
        "powerpc64",
        "s390x",
        "sh"; ".S",
        "x32",
        "x86_64"
    }

    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    fs::create_dir_all("../../target/include").expect("failed to create include directory");
    cbindgen::generate(crate_dir)
        .expect("failed to generate bindings")
        .write_to_file("../../target/include/setjmp.h");
}
