# What is the Controller?

The **Controller** is a vital component of Atomic Cloud. It acts as the central management server by:

- **Overseeing Nodes:**  
  It monitors and manages the nodes.

- **Managing Servers:**  
  It is responsible for initiating and supervising the servers launched by the cloud.

---

# Docker Installation (Recommended)

The most straightforward method to install the controller is by utilizing a Docker image. Follow the steps below to set it up using Docker Compose:

## Step 1: Create the `docker-compose.yml` File
First, use a text editor to create the `docker-compose.yml` file:
```bash
nano docker-compose.yml
```
Next, add the following content to the file:
```yaml
services:
  controller:
    image: ghcr.io/httprafa/atomic-cloud:latest
    ports:
      - "8080:8080"
    environment:
      - PTERODACTYL=true # Enable Pterodactyl plugin installation
      - LOCAL=true       # Enable Local plugin installation
    volumes:
      - ./certs:/app/certs
      - ./configs:/app/configs
      - ./groups:/app/groups
      - ./logs:/app/logs
      - ./nodes:/app/nodes
      - ./plugins:/app/plugins
      - ./users:/app/users
      - ./data:/app/data
```

## Step 2: Start the Container
To start the container, execute the following command:
```bash
docker compose up
```

---

# Manual Installation

Follow the steps below to manually install Atomic Cloud.

## Step 1: Download the CLI and Controller

 Download the latest release from our [GitHub releases page](https://github.com/HttpRafa/atomic-cloud/releases).

 **Controller:** Choose the version that corresponds to the operating system where the Controller will run.

 **CLI:** Choose the version that matches the operating system on your local machine, from which you will manage the cloud.

## Step 2: Start the Controller

1. Open a terminal and navigate to the directory where the Controller is located.
2. Start the Controller.
3. **Important:** After startup, note the authentication token that is displayed. You will need this token later.  
   If you lose the token, you can retrieve it from our [token retrieval guide](../usage/controller/retrieve_token.md).

## Step 3: Download and Install the Plugin

1. Download the latest plugin version from our [GitHub releases page](https://github.com/HttpRafa/atomic-cloud/releases).
2. Place the plugin file into the `plugins` folder.
3. Restart the Controller to load the new plugin.

## Step 4: Start the CLI

1. Open a terminal on your local device where you want to control the cloud.
2. Start the CLI application.
3. When prompted, select **"Add new controller"** and follow the on-screen instructions to complete the setup.