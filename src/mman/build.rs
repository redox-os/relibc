extern crate cbindgen;

use std::{env, fs};

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    fs::create_dir_all("../../target/include").expect("failed to create include directory");
    cbindgen::generate(crate_dir)
      .expect("failed to generate bindings")
      .write_to_file("../../target/include/mman.h");
}
