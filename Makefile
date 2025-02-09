.PHONY: run run-controller build build-controller build-wrapper build-plugins clean fix

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
	cargo clippy --fix --allow-dirty --allow-staged --all-features

## Build target
build: build-controller build-cli build-wrapper build-plugins

## Run target
run: run-controller

## Run controller
run-controller:
	$(MKDIR) $(RUN_DIR) $(SEP) cd $(RUN_DIR) $(SEP) cargo run -p controller --all-features -- $(CONTROLLER_ARGS)

## Run cli
run-cli:
	$(MKDIR) $(RUN_DIR) $(SEP) cd $(RUN_DIR) $(SEP) cargo run -p cli --all-features -- $(CLI_ARGS)

## Build controller target
build-controller:
	cargo build -p controller --all-features --release

## Build cli target
build-cli:
	cargo build -p cli --all-features --release

## Build wrapper target
build-wrapper:
	cargo build -p wrapper --all-features --release

## Build plugins target
build-plugins:
	$(SETENV) RUSTFLAGS="$(WASM_RUSTFLAGS)"
	cargo build -p pterodactyl --target $(WASM_TARGET) --release
	cargo build -p local --target $(WASM_TARGET) --release

# Create plugin directory if it doesn't exist
$(PLUGIN_DIR):
	$(MKDIR) $(PLUGIN_DIR)