# Permissions

WASM plugins operate within a sandbox environment, meaning they have very limited access to the host system by default. Both the WASI standard and the Controller provide APIs to facilitate integration with network or local resources. However, you must explicitly grant the necessary permissions to each plugin to enable such access.

You can configure these permissions in the `configs/wasm-plugins.toml` file. For example:

1. **Open the Configuration File:**

```bash
nano wasm-plugins.toml
```

2. **Configure Plugin Permissions:**

```toml
# This configuration is crucial for granting the plugins their required permissions
# https://httprafa.github.io/atomic-cloud/usage/plugins/wasm/permissions/

[[plugins]]
name = "local"
inherit_stdio = false
inherit_args = false
inherit_env = false
inherit_network = false
allow_ip_name_lookup = false
allow_http = false
allow_process = true
allow_remove_dir_all = true
mounts = []

[[plugins]]
name = "pelican"
inherit_stdio = false
inherit_args = false
inherit_env = false
inherit_network = true
allow_ip_name_lookup = true
allow_http = true
allow_process = false
allow_remove_dir_all = false
mounts = []
```

### Explanation of Configuration Options

- **inherit_stdio, inherit_args, inherit_env:**  
  These options control whether the plugin inherits the standard I/O streams, command-line arguments, and environment variables from the host.

- **inherit_network:**  
  Determines if the plugin can access the network interfaces of the host.

- **allow_ip_name_lookup:**  
  Enables the plugin to perform DNS lookups.

- **allow_http:**  
  Permits the plugin to make HTTP requests.

- **allow_child_processes:**  
  Specifies whether the plugin can spawn child processes.

- **mounts:**  
  Configures file or socket mounts between the host and the plugin environment. For instance, mounting the Docker socket allows a plugin to interact with Docker on the host.

This setup ensures that plugins run securely while still having the flexibility to integrate with local or network resources as required.