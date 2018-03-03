set -ex

cargo build
cargo build --manifest-path crt0/Cargo.toml
cd tests
make clean
make run
