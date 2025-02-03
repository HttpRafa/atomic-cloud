# Channels

Channels are a pub/sub (publish/subscribe) messaging system within Atomic Cloud designed for efficient, secure, real‑time communication between servers. They work similarly to MQTT and are ideal for implementing features such as party systems, player notifications, and any functionality that requires inter‑server messaging.

## Table of Contents

- [What Are Channels?](#what-are-channels)
- [Prerequisites](#prerequisites)
- [Receiving Messages](#receiving-messages)
- [Sending Messages](#sending-messages)
- [Conclusion](#conclusion)

## What Are Channels?

Channels provide a central, consistent method for sending and receiving messages across distributed servers. By using channels, you ensure data integrity and security throughout your application, making it easier to orchestrate communication between various services or components.

## Prerequisites

Before using channels, make sure to:

- **Choose a Unique Channel Name:**  
  The channel name serves as the unique identifier for message routing.

- **Subscribe to the Channel:**  
  Only servers or clients that subscribe to a channel will receive its messages.

- **Unsubscribe When Needed:**  
  Subscribers can unsubscribe at any time to stop receiving messages.

## Receiving Messages

To receive messages from a channel, subscribe and register a message handler using the JVM API.

### Example

```java
// Subscribe to the "party/invites" channel
Cloud.channels().subscribe("party/invites");

// Register a handler to process incoming messages on "party/invites"
Cloud.channels().registerHandler("party/invites", (message) -> {
    // Define your message handling logic here
    System.out.println("Received message: " + message);
});
```

Once subscribed, every message sent to the channel will trigger the registered handler.

## Sending Messages

Sending messages is simple. Use the API to send a message to a channel, and all subscribers will receive it.

### Example

```java
// Send a message to the "party/invites" channel
Cloud.channels().sendMessage("party/invites", "test");
```

In this example, the string `"test"` is broadcast to all clients subscribed to the "party/invites" channel.

## Conclusion

Channels in Atomic Cloud offer a robust and secure method for inter‑server communication, allowing you to easily implement real‑time features with minimal setup. By following the steps above, you can integrate channels into your project to enable seamless, efficient messaging between distributed services.