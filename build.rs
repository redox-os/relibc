extern crate cbindgen;
extern crate cc;

use std::{env, fs, fs::DirEntry, path::Path};

// include src/header directories that don't start with '_'
fn include_dir(d: &DirEntry) -> bool {
    d.metadata().map(|m| m.is_dir()).unwrap_or(false)
        && d.path()
            .iter()
            .nth(2)
            .map_or(false, |c| c.to_str().map_or(false, |x| !x.starts_with("_")))
}

fn get_target() -> String {
    env::var("TARGET").unwrap_or(
        option_env!("TARGET").map_or("x86_64-unknown-redox".to_string(), |x| x.to_string()),
    )
}

fn generate_bindings(cbindgen_config_path: &Path, prefix: &str) {
    let relative_path = cbindgen_config_path
        .strip_prefix(prefix)
        .ok()
        .and_then(|p| p.parent())
        .and_then(|p| p.to_str())
        .unwrap()
        .replace("_", "/");
    let header_path = Path::new("target/include")
        .join(&relative_path)
        .with_extension("h");
    let mod_path = cbindgen_config_path.with_file_name("mod.rs");
    let config = cbindgen::Config::from_file(cbindgen_config_path).unwrap();
    cbindgen::Builder::new()
        .with_config(config)
        .with_src(mod_path)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(header_path);
}

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let target = get_target();

    // Generate C includes
    // - based on contents of src/header/**
    // - headers written to target/include
    fs::read_dir(&Path::new("src/header"))
        .unwrap()
        .into_iter()
        .filter_map(Result::ok)
        .filter(|d| include_dir(d))
        .map(|d| d.path().as_path().join("cbindgen.toml"))
        .filter(|p| p.exists())
        .for_each(|p| {
            println!("cargo:rerun-if-changed={:?}", p.parent().unwrap());
            println!("cargo:rerun-if-changed={:?}", p);
            println!("cargo:rerun-if-changed={:?}", p.with_file_name("mod.rs"));
            generate_bindings(&p, "src/header");
        });

    fs::read_dir(&Path::new("src/libm"))
        .unwrap()
        .into_iter()
        .filter_map(Result::ok)
        .filter(|d| include_dir(d))
        .map(|d| d.path().as_path().join("cbindgen.toml"))
        .filter(|p| p.exists())
        .for_each(|p| {
            println!("cargo:rerun-if-changed={:?}", p.parent().unwrap());
            println!("cargo:rerun-if-changed={:?}", p);
            println!("cargo:rerun-if-changed={:?}", p.with_file_name("mod.rs"));
            generate_bindings(&p, "src/libm");
        });

    println!("cargo:rerun-if-changed=src/c");

    let mut cc_builder = &mut cc::Build::new();

    cc_builder = cc_builder
        .flag("-nostdinc")
        .flag("-nostdlib")
        .include(&format!("{}/include", crate_dir))
        .include(&format!("{}/target/include", crate_dir));

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
