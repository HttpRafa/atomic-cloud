# Modular Backend

Atomic Cloud features a versatile "driver system" that enables the cloud to utilize various methods to initiate the server. Examples of supported backends include Pterodactyl, Docker, and traditional servers (similar to CloudNet).

## Supported Languages for Driver Development

Drivers are implemented using WebAssembly and adhere to the WASI standard, which imposes certain compatibility constraints. It is highly recommended to develop drivers in Rust or utilize TeaVM for Java to ensure optimal performance and compatibility.

## Currently Available Drivers

1. Pterodactyl (with plans to support Pelican in the future)