extern crate cbindgen;

use cbindgen::{Builder, Config};
use std::env;
use std::fs;
use std::path::Path;

// Modules and crates that we should not generate a header for
const IGNORE: [&str; 4] = ["lib.rs", "crt0", "platform", "todo"];

// Create the parent directory for a header file we are about to generate
fn create_parent(path: &Path) {
    match path.parent() {
        Some(path) => {
            fs::create_dir_all(format!("./target/include/{}", path.to_str().unwrap()))
                .expect("failed to create include directory");
        },
        _ => (),
    }
}

// Recursively attempt to find crates and modules and generate headers
// for them.
fn build_subdirs<P: AsRef<Path>>(crate_dir: P, root_dir: &str) {
    let entries = fs::read_dir(crate_dir).unwrap();
    for entry in entries {
        // Generate some common variables we will use
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        let path = entry.path();
        let target_path = Path::new("./target/include/");
        // Do not attept to generate a header for a module or crate we
        // should ignore
        if !IGNORE.contains(&&*file_name) {
            // Find the path of the file relative to CARGO_MANIFEST_DIR/src
            let relative_location = path.strip_prefix(root_dir).unwrap();
            // Build the path to install the header. It should be
            // ./target/include/<relative path to src dir>.
            let target_location = target_path.join(relative_location.with_extension("h"));
            if file_type.is_dir() {
                // If the current path being scanned is a directory attempt to find
                // a mod.rs file or a Cargo.toml file.
                let mut cargo_toml = entry.path();
                cargo_toml.push("Cargo.toml");
                let mut mod_rs = entry.path();
                mod_rs.push("mod.rs");

                if Path::new(&cargo_toml).exists() {
                    // We found a Cargo.toml file. This is a crate. Just call
                    // cbindgen::generate on the crate.
                    create_parent(&relative_location);
                    cbindgen::generate(path.clone())
                        .expect("failed to generate bindings")
                        .write_to_file(target_location);
                } else if Path::new(&mod_rs).exists() {
                    // TODO: Add support for this
                    panic!("The build script does not currently support directory modules");
                } else {
                    // This is neither a module nor a crate. Recurse into this directory
                    // and attempt to find more crates or modules that need generated
                    // headers.
                    build_subdirs(path.clone(), root_dir);
                }
            } else if file_name.ends_with(".rs") {
                // We found a module that isn't a directory. In order to correctly generate
                // a header we need a corresponding cbindgen.toml. Currently the cbindgen
                // config is expected to be in the same directory as the source and have the
                // same name with the extension ".toml".
                create_parent(&relative_location);
                let config_path = path.with_extension("toml");
                if config_path.exists() {
                    // We found a config file. Parse the config and generate the
                    // header.
                    let config = Config::from_file(config_path.to_str().unwrap()).unwrap();
                    let bindings = Builder::new()
                        .with_config(config)
                        .with_src(&path)
                        .generate();
                    match bindings {
                        Ok(bindings) => {
                            bindings.write_to_file(target_location);
                        }
                        Err(e) => {
                            panic!("Failed to generate bindings for {}: {:?}", file_name, e);
                        }
                    }
                } else {
                    // Without a config file there is nothing we can do.
                    panic!("Found module without corresponding cbindgen config")
                };
            }
        }
    }
}

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    fs::create_dir_all("./target/include").expect("failed to create include directory");
    let root_dir = format!("{}/src", crate_dir);
    build_subdirs(&root_dir, &root_dir);
}
