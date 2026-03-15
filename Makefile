.PHONY: run run-controller build build-host build-plugins clean fix

# Configuration
WASM_RUSTFLAGS = -Z wasi-exec-model=reactor
WASM_TARGET = wasm32-wasip2

# Directories
RUN_DIR = run
OLD_RUN_DIR = run.old
PLUGIN_DIR = $(RUN_DIR)/plugins/wasm

# Arguments
CONTROLLER_ARGS = "--debug"
CLI_ARGS = "--debug"

# OS detection
ifeq ($(OS),Windows_NT)
    RM = cmd /C del /S /Q
    MKDIR = mkdir
    CP = xcopy /E /I /Y
    SEP = &
else
    RM = rm -rf
    MKDIR = mkdir -p
    CP = cp -r
    SEP = ;
endif

# Targets

## Clean target
clean:
    $(CP) $(RUN_DIR) $(OLD_RUN_DIR)
    $(RM) $(RUN_DIR)
    cargo clean

## Fix target
fix:
    cargo clippy --fix --allow-dirty --allow-staged --all-features
    cargo fmt

## Build target (Uses -j2 to run host and wasm builds simultaneously)
build: 
    $(MAKE) -j2 build-host build-plugins

## Run target
run: run-controller

## Run controller
run-controller:
    $(MKDIR) $(RUN_DIR) $(SEP) cd $(RUN_DIR) $(SEP) cargo run -p controller --all-features -- $(CONTROLLER_ARGS)

## Run cli
run-cli:
    $(MKDIR) $(RUN_DIR) $(SEP) cd $(RUN_DIR) $(SEP) cargo run -p cli --all-features -- $(CLI_ARGS)

## Build host binaries (controller, cli, wrapper) together
build-host:
    cargo build -p controller -p cli -p wrapper --all-features --release

## Build plugins together
build-plugins:
ifeq ($(OS),Windows_NT)
    set "RUSTFLAGS=$(WASM_RUSTFLAGS)" && cargo build -p pelican -p local -p cloudflare --target $(WASM_TARGET) --release
else
    RUSTFLAGS="$(WASM_RUSTFLAGS)" cargo build -p pelican -p local -p cloudflare --target $(WASM_TARGET) --release
endif

# Create plugin directory if it doesn't exist
$(PLUGIN_DIR):
    $(MKDIR) $(PLUGIN_DIR)