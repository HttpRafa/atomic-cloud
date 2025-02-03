# Modular Backend

Atomic Cloud is built on a versatile driver system that abstracts and streamlines the process of initiating servers. This modular approach allows the platform to support various backends, ensuring flexibility and scalability. Currently, Atomic Cloud supports several backend types, including:

- **Pterodactyl**
- **Docker**
- **Traditional Servers** (similar to CloudNet)

## Supported Languages for Driver Development

Plugins for Atomic Cloud are implemented using WebAssembly and adhere to the WASI standard. Due to compatibility constraints inherent to WASI, it is highly recommended to develop Plugins in one of the following languages to ensure optimal performance and compatibility:

- **Rust**
- **Java** (using [TeaVM](https://www.teavm.org/) for transpilation)

## Currently Available Plugins

- **Pterodactyl Driver:** Enables integration with the Pterodactyl backend.  
  *Future plans include support for the Pelican backend.*