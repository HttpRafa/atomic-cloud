cargo build --target wasm32-wasi --release
mkdir -p ../../run/drivers/wasm/
wasm-tools component new ./target/wasm32-wasi/release/pelican.wasm \
    -o ../../run/drivers/wasm/pelican.wasm --adapt ../wasi_snapshot_preview1.reactor.wasm
