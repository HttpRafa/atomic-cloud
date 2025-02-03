# Templates

Templates in Atomic Cloud are pre-defined structures that help you quickly create and manage resources. Think of them as blueprints—similar to a Minecraft server setup—that standardize the configuration and behavior of various components within your cloud environment.

## What are Templates?

Templates serve as blueprints for your groups. They define the desired configuration of resources, whether that’s a virtual machine, storage account, networking component, or even a fully configured Minecraft server setup. By using templates, you can ensure consistency and repeatability in your groups.

## How Do Templates Work?

Templates work by describing the desired state of your resources in a structured file, typically using JSON, YAML, or TOML. When you deploy a template, the Atomic Cloud Controller reads this file and provisions the resources exactly as specified.

### Key Components of a Template

1. **Parameters**:  
   These are inputs that allow you to customize the group. For example, you might specify the server name, memory allocation, or game-specific settings like port numbers and world seeds.

2. **Resources**:  
   These define the actual components to be created. In a Minecraft server template, resources could include the server jar file, configuration files, mods, or plugins required to run the server.

3. **Outputs**:  
   These are values returned after the deployment is complete, such as resource IDs, connection URLs, or IP addresses that you might need to connect to your server.

### Example Template Structure

Imagine you want to deploy a Minecraft server. Your template might look like this:

```
templates/
    papermc/
        bukkit.yml
        eula.txt
        plugins/
            paper-client.jar
        prepare/
            unix.sh
            windows.ps1
        server.jar
        server.properties
        startup/
            unix.sh
            windows.ps1
        template.toml
```

### Example Template File

Below is an example of a TOML-based template file for a Minecraft server (using PaperMC as an example):

```toml
# filepath: /templates/papermc/template.toml

[parameters]
# Customize your server settings
serverName = "MyMinecraftServer"
serverPort = "25565"
maxPlayers = 20

[resources]
# Define the Minecraft server resource
[[resources.minecraftServer]]
type = "Minecraft.Server"
version = "1.18.1"
name = "{parameters.serverName}"
port = "{parameters.serverPort}"
maxPlayers = "{parameters.maxPlayers}"
jarFile = "server.jar"
eula = "eula.txt"
configFiles = [
    "bukkit.yml",
    "server.properties"
]
plugins = [
    "plugins/paper-client.jar"
]
startupScripts = [
    "startup/unix.sh",
    "startup/windows.ps1"
]

[outputs]
# Output values after deployment
serverUrl = "http://{parameters.serverName}.atomiccloud.example.com:{parameters.serverPort}"
```

### How This Template Helps

- **Consistency:**  
  Every time you deploy the template, you get the same server setup, ensuring a predictable environment.
  
- **Customization:**  
  Parameters allow you to adjust key settings (like server name, port, and maximum players) without modifying the underlying resource definitions.

- **Simplicity:**  
  With a pre-defined structure, you can quickly spin up a new Minecraft server with all the necessary configurations and files, reducing manual setup time.

By using templates, you can streamline the process of deploying and managing resources in Atomic Cloud, ensuring that your infrastructure is consistent, scalable, and easy to manage—just like setting up your ideal Minecraft server.