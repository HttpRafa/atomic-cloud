# Welcome
Atomic Cloud is a cloud that was primarily developed for the game Minecraft, but it is possible to integrate other games with Atomic Cloud.

## Project State
The cloud is still very early in development, so some of the features listed below are not available. But they will be available in the first full release.

## What makes it special?
### No proxy
Atomic Cloud is designed from the ground up with the new Minecraft transfer packet which means there is no need to use proxy software like Velocity or Bungeecord.

#### Why is this better?
The primary benefit of removing a proxy is the reduced latency and the ability to easily split players based on their region. Furthermore, it is almost impossible to crash the entire network using crashing methods as there is no single point of failure. If you manage to crash the lobby, a new one starts on a new random port that must first be found before a new attack can be launched.

### Modular backend
Atomic Cloud has a "driver system" that allows the cloud to use whatever to start the server. Examples of backends are: Pterodactyl, Docker or normal servers (similar to CloudNet).
#### What language can i use to write a driver?
-> The plugins use WebAssembly and the WASI standard, so compatibility is very limited. My recommendation is to write the drivers in Rust or use TeaVM for Java.
#### Currently existing drivers
1. Pterodactyl (Pelican in the future)

### CLI Application
In addition, the cloud has a CLI application that, like kubectl, allows you to control the cloud without having to log into an SSH window every time you want to make a small change.

## What API does the cloud have?
1. To enable communication between servers, the cloud has a channel system. For example, data can be sent from the game servers to the lobby via channels.
2. It also has API to start servers and move players.