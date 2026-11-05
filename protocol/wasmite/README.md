# Wasmite plugins

Wasmite is the second plugin runtime of the controller, next to the
original `wasm` runtime (`protocol/wasm/`). The interface is defined in
[`plugin.wit`](plugin.wit).

## Execution model

Every exported function runs on a **fresh instance** of the plugin, which is
thrown away when the call returns. Nothing kept in guest memory survives
between calls, and calls may run **concurrently**.

Everything that has to outlive a call is owned by the host:

| What | Where | Notes |
| --- | --- | --- |
| State | `kv` | Persistent, per plugin. Use `compare-and-swap` for every read-modify-write. |
| Child processes | `process` | Addressed by an id the plugin chooses. Output is buffered on the host. |
| Server screens | `process` | `node.start` returns the id of the process whose output is the screen. |
| Stop completion | `server.stopped` | Call it once a server stopped through `node.stop` is down. |

Each call is limited to 30 seconds and 256 MiB of memory. Long-running work
(for example a prepare script that downloads files) has to be started as a
process and checked on in `tick`.

Benefits compared to the `wasm` runtime:

- A trap, a leak or a stuck call only affects that one call; a timed-out
  call can be cancelled safely because its instance is discarded anyway.
- Calls into one plugin no longer queue behind a single store lock.
- Plugin state is persisted, and processes left behind by a crashed
  controller are killed on the next start (on Linux, after checking that the
  recorded pid still runs the program that was started).

## Installing

Wasmite plugins are loaded from `plugins/wasmite/*.wasm`; `plugins/*.wasm`
stays reserved for the `wasm` runtime. Permissions are configured in the
same `configs/wasm-plugins.toml` file:

- `ALLOW_PROCESS` is required for the `process` spawn functions
- `ALLOW_REMOVE_DIR_ALL` is required for `fs.remove-dir-all`

The controller has to be built with the `wasmite-plugins` feature.

## Host files

The host keeps its own files in `data/<plugin>/.wasmite/`:

- `kv.toml`: the plugin's kv store
- `processes.toml`: pids of running processes, used to clean up after a crash

## Building a plugin

```sh
RUSTFLAGS="-Z wasi-exec-model=reactor" cargo build -p local --target wasm32-wasip2 --release
```

See `plugins/local` for a complete plugin.
