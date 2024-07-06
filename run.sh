#/usr/bin/env bash
mkdir -p run/
pushd run
RUSTFLAGS="-Z wasi-exec-model=reactor" cargo +nightly build -p pterodactyl --target wasm32-wasip1 --release
mkdir -p ../run/drivers/wasm/
wasm-tools component new ../target/wasm32-wasip1/release/pterodactyl.wasm \
    -o ../run/drivers/wasm/pterodactyl.wasm --adapt ../drivers/files/wasi_snapshot_preview1.reactor.wasm
cargo run -p controller --all-features -- "$@"
popd