extern crate cc;

use std::{env, fs};

fn get_target() -> String {
    env::var("TARGET").unwrap_or(
        option_env!("TARGET").map_or("x86_64-unknown-redox".to_string(), |x| x.to_string()),
    )
}

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let target = get_target();

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
