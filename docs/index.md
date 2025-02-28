# Welcome

Welcome to the **Atomic Cloud** project! Please note that the cloud is currently in its **early development stages**, and you might encounter occasional bugs. If you experience any issues, please report them by opening an issue on our [GitHub repository](https://github.com/HttpRafa/atomic-cloud).

## Documentation Status 🚧

This documentation is a work in progress. We apologize for any grammatical errors or incomplete sections. We welcome contributions from the commservery—if you’d like to help improve the documentation, please consider submitting a Pull Request.

## Features
### Command Line Interface (CLI) Application

The Atomic Cloud platform includes a powerful Command Line Interface (CLI) application that simplifies the management and control of your cloud resources—similar in functionality to `kubectl`. This tool is designed to streamline your workflow by eliminating the need for frequent SSH logins when making minor adjustments. [More details](cli/)

### Robust APIs

The Atomic Cloud platform features a sophisticated API suite designed to enhance server communication and overall system efficiency. [More detail](api/)

### Modular Backend

Atomic Cloud is built on a versatile plugin system that abstracts and streamlines the process of initiating servers. This modular approach allows the platform to support various backends, ensuring flexibility and scalability. [Take a look](plugins/)

### No Proxy Usage

Atomic Cloud is engineered using the latest Minecraft transfer packet technology, eliminating the need for traditional proxy software such as Velocity or BungeeCord. This innovative approach provides several key advantages:

#### Reduced Latency

By removing the proxy layer, Atomic Cloud minimizes network overhead, significantly reducing latency. This results in a smoother and more responsive gaming experience for players.

#### Regional Player Distribution

Without a centralized proxy, players can be seamlessly distributed based on their geographical regions. This not only improves server performance but also enhances the overall player experience by connecting them to the nearest available server.

#### Enhanced Network Stability

Eliminating the proxy creates a more resilient network architecture. Without a single point of failure, the system is better protected against crashes. For instance, if a lobby experiences a failure, a new lobby is automatically initiated on a random port, making it more challenging for attackers to target and disrupt the network consistently.

Atomic Cloud's no-proxy design ensures a robust, efficient, and secure environment, providing significant performance improvements and increased reliability for all players.