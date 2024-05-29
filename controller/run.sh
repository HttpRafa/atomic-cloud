#/usr/bin/env bash
pushd scripts
./build_drivers.sh
popd

pushd run
cargo run -p controller --all-features
popd