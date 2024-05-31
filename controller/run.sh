#/usr/bin/env bash
pushd scripts
./build-pelican.sh
popd

pushd run
cargo run -p controller --all-features -- "$@"
popd