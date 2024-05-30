export RUSTFLAGS="-Z wasi-exec-model=reactor"
cargo +nightly build -p pelican --target wasm32-wasip1 --release
mkdir -p ../run/drivers/wasm/
wasm-tools component new ../target/wasm32-wasip1/release/pelican.wasm \
    -o ../run/drivers/wasm/pelican.wasm --adapt ../drivers/files/wasi_snapshot_preview1.reactor.wasm