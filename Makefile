.PHONY: run run-controller build build-controller build-wrapper build-drivers create-components clean fix

# Configuration
WASM_RUSTFLAGS = -Z wasi-exec-model=reactor
WASM_TARGET = wasm32-wasip1
WASM_COMPONENT = target/wasm32-wasip1/release/pterodactyl.wasm
WASM_ADAPT_COMPONENT = drivers/files/wasi_snapshot_preview1.reactor.wasm

# Directories
RUN_DIR = run
OLD_RUN_DIR = run.old
DRIVER_DIR = $(RUN_DIR)/drivers/wasm

# Arguments
CONTROLLER_ARGS = ""

# OS detection
ifeq ($(OS),Windows_NT)
    RM = cmd /C del /S /Q
    MKDIR = mkdir
    CP = xcopy /E /I /Y
    SEP = &
	SETENV = set
else
    RM = rm -rf
    MKDIR = mkdir -p
    CP = cp -r
    SEP = ;
	SETENV = export
endif

# Targets

## Clean target
clean:
	$(CP) $(RUN_DIR) $(OLD_RUN_DIR)
	$(RM) $(RUN_DIR)
	cargo clean

## Fix target
fix:
	cargo fmt
	cargo clippy --fix --allow-dirty --all-targets --all-features

## Build target
build: build-controller build-wrapper build-drivers create-components

## Run target
run: build run-controller

## Run controller
run-controller:
	$(MKDIR) $(RUN_DIR) $(SEP) cd $(RUN_DIR) $(SEP) cargo run -p controller --all-features -- $(CONTROLLER_ARGS)

## Build controller target
build-controller:
	cargo build -p controller --all-features --release

## Build wrapper target
build-wrapper:
	cargo build -p wrapper --all-features --release

## Build drivers target
build-drivers:
	$(SETENV) RUSTFLAGS="$(WASM_RUSTFLAGS)"
	cargo +nightly build -p pterodactyl --target $(WASM_TARGET) --release

## Component target
create-components: $(DRIVER_DIR)
	wasm-tools component new $(WASM_COMPONENT) -o $(DRIVER_DIR)/pterodactyl.wasm --adapt $(WASM_ADAPT_COMPONENT)

# Create driver directory if it doesn't exist
$(DRIVER_DIR):
	$(MKDIR) $(DRIVER_DIR)