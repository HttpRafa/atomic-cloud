.PHONY: run run-controller build-drivers create-components clean

# Configuration
WASM_RUSTFLAGS = -Z wasi-exec-model=reactor
WASM_TARGET = wasm32-wasip1
WASM_COMPONENT = target/wasm32-wasip1/release/pterodactyl.wasm
WASM_ADAPT_COMPONENT = drivers/files/wasi_snapshot_preview1.reactor.wasm

# Directories
RUN_DIR = run
DRIVER_DIR = $(RUN_DIR)/drivers/wasm

# Arguments
CONTROLLER_ARGS = ""

# Targets

## Clean target
clean:
	rm -rf $(RUN_DIR)
	cargo clean

## Build target
build: build-controller build-wrapper build-drivers create-components

## Run target
run: build-drivers create-components run-controller

## Run controller
run-controller:
	(cd $(RUN_DIR); cargo run -p controller --all-features -- $(CONTROLLER_ARGS))

## Build controller target
build-controller:
	cargo build -p controller --all-features --release

## Build wrapper target
build-wrapper:
	cargo build -p wrapper --all-features --release

## Build drivers target
build-drivers:
	RUSTFLAGS="$(WASM_RUSTFLAGS)" cargo +nightly build -p pterodactyl --target $(WASM_TARGET) --release

## Component target
create-components: $(DRIVER_DIR)-directory
	wasm-tools component new $(WASM_COMPONENT) -o $(DRIVER_DIR)/pterodactyl.wasm --adapt $(WASM_ADAPT_COMPONENT)

# Create run directory if it doesn't exist
$(RUN_DIR)-directory:
	mkdir -p $(RUN_DIR)

# Create driver directory if it doesn't exist
$(DRIVER_DIR)-directory:
	mkdir -p $(DRIVER_DIR)