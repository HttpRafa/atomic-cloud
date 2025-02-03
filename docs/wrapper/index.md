# Wrapper

The **Wrapper** is a process that acts as an intermediary between the **Controller** and a target game server. This solution is useful when direct modifications to the game server code are not feasible. A common use case is for environments where the game server, such as a **Vanilla Minecraft server**, cannot be modified directly, but the server needs to communicate with a controlling system that requires specific behaviors (e.g., sending a heartbeat message).

### Purpose

The Wrapper ensures the game server meets certain requirements, like sending regular heartbeat messages to the **Controller**. For instance, the default heartbeat interval is every 15 seconds. If the server fails to send this heartbeat, it is considered "dead" or "stopped" by the **Controller**. Since the game server might not be able to fulfill this task directly, the Wrapper monitors the server process and simulates the required heartbeat functionality.

### Functionality

- **Process Monitoring**: The Wrapper starts and monitors the game server (child process). While the server is running, the Wrapper intercepts its standard input/output (stdin/stdout) streams.
- **Heartbeat Simulation**: Instead of the server itself sending the heartbeat, the Wrapper simulates the process by sending a heartbeat message at the defined interval. If the server fails to provide the heartbeat within the expected time, the **Controller** considers the server to be unresponsive.

### Caveats

- **Limited Functionality**: The Wrapper currently supports basic functionality, such as sending user transfer commands via stdin. However, some server functionalities may be restricted or altered, especially features like server transfers. For example, the Wrapper can only send user transfer input to the server's stdin stream, enabling user redirection to different servers.
- **Unmaintained State**: The Wrapper is in a somewhat unmaintained state at the moment. While it works in basic scenarios, there are plans to improve its capabilities and ensure compatibility with a wider range of game server environments.