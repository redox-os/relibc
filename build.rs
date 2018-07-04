extern crate cc;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    cc::Build::new()
        .flag("-nostdinc")
        .flag("-nostdlib")
        .flag("-I")
        .flag(&format!("{}/include", crate_dir))
        .flag("-fno-stack-protector")
        .file("src/c/dlmalloc.c")
        .file("src/c/fcntl.c")
        .file("src/c/stdio.c")
        .compile("relibc_c");

    println!("cargo:rustc-link-lib=static=relibc_c");
}
