set -ex

cargo build
cargo build --manifest-path src/crt0/Cargo.toml

cd openlibm
make
cd ..

cd tests
make clean
make run
