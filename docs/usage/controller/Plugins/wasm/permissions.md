# Permissions

WASM Plugins operate within a sandbox environment, which means they have very limited access to the host system by default. The WASI standard and the controller provide APIs to enable integration with network or local resources. However, you must grant the appropriate permissions to the Plugin you install to allow such access.

You can configure the permissions in the `configs/wasm.toml` file:

```bash
nano wasm.toml
```

```toml
# This configuration is crucial for granting the plugins their required permissions
# https://httprafa.github.io/atomic-cloud/controller/plugins/wasm/permissions/

[[Plugins]]
name = "local"
inherit_stdio = true
inherit_args = true
inherit_env = true
inherit_network = true
allow_ip_name_lookup = true
allow_http = true
allow_child_processes = true

[[plugins.mounts]]
host = "/var/run/docker.sock"
guest = "/var/run/docker.sock"

[[plugins]]
name = "pterodactyl"
inherit_stdio = false
inherit_args = false
inherit_env = false
inherit_network = true
allow_ip_name_lookup = true
allow_http = true
allow_child_processes = false
mounts = []
```