# What is a template?

A template in the CloudNet world is a predefined state of a server. When a new server is started, this template is copied to the new server's directory. This allows for the creation of specific templates for different types of servers, such as a lobby server or a game server. For example, you can have a template for a Minecraft lobby server and another for a Bedwars game server.

## Example File (for PaperMC)

```toml
description = "PaperMC improves Minecraft's ecosystem with fast, secure software and an expanding plugin API, providing quick releases and helpful support as the most widely used, performant, and stable software available."
version = "0.1.0"
authors = ["HttpRafa"]

exclusions = ["template.toml", "prepare/"]

shutdown = "stop"

[environment]
PROJECT = "paper"
VERSION = "latest"
BUILD_NUMBER = "latest"
SERVER_JARFILE = "server.jar"

[prepare.unix]
command = "bash"
args = ["-c", "chmod +x prepare/unix.sh && ./prepare/unix.sh"]

[prepare.windows]
command = "powershell.exe"
args = ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "prepare/windows.ps1"]

[startup.unix]
command = "bash"
args = ["-c", "chmod +x startup/unix.sh && ./startup/unix.sh"]

[startup.windows]
command = "powershell.exe"
args = ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "startup/windows.ps1"]
```