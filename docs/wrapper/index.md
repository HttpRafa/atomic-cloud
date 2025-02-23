# Wrapper

The **Wrapper** is an intermediary process that bridges the **Controller** and a target game server. It is especially useful when direct modifications to the game server’s code are not feasible. For example, in environments where a **Vanilla Minecraft server** cannot be altered to support custom behaviors, the Wrapper allows the server to communicate with the controlling system by simulating required functions.

## Purpose

The primary purpose of the Wrapper is to ensure that the game server meets specific operational requirements imposed by the Controller. A common requirement is the need to send regular heartbeat messages. For instance, the Controller expects a heartbeat at 15-second intervals. If the server does not send a heartbeat within that window, it is deemed "dead" or "stopped." Since some game servers cannot be modified to provide this functionality natively, the Wrapper monitors the server process and simulates the heartbeat functionality on its behalf.

## Functionality

- **Process Monitoring**:  
  The Wrapper launches and continuously monitors the game server (its child process). During runtime, it intercepts the server’s standard input and output streams to manage communication.

- **Heartbeat Simulation**:  
  Rather than requiring the server to send its own heartbeat, the Wrapper periodically sends a simulated heartbeat message at the defined interval. If the Controller does not receive a heartbeat within the expected timeframe, it marks the server as unresponsive.

## Caveats

- **Limited Functionality**:  
  Currently, the Wrapper supports basic functions such as sending user transfer commands via the server’s standard input. Certain advanced functionalities (e.g., server transfers) may be limited or altered. For example, while the Wrapper can forward user transfer commands to the server, it cannot modify more complex behaviors inherent to the server software.

- **Maintenance Status**:  
  The Wrapper is in a somewhat unmaintained state at present. Although it works well for basic scenarios, improvements are planned to enhance its capabilities and broaden compatibility with a wider range of game server environments.