extern crate cc;

use std::{env, fs};

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    cc::Build::new()
        .flag("-nostdinc")
        .flag("-nostdlib")
        .include(&format!("{}/include", crate_dir))
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
