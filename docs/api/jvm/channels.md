# Channels

Channels are a pub/sub (publish/subscribe) messaging system within Atomic Cloud designed for efficient, secure, real‑time communication between servers. They work similarly to MQTT and are ideal for implementing features such as party systems, player notifications, and any functionality that requires inter‑server messaging.

## Table of Contents

- [What Are Channels?](#what-are-channels)
- [Prerequisites](#prerequisites)
- [Receiving Messages](#receiving-messages)
- [Sending Messages](#sending-messages)

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
var channel = Cloud.channels().subscribeToStrings("testChannel");
channel.handler(message -> {
  // A message was received from the channel.
});
channel.errorHandler(throwable -> {
  // Oh no! An error occurred while trying to receive a message from the channel.
});
```

Once subscribed, every message sent to the channel will trigger the registered handler.

## Sending Messages

Sending messages is simple. Use the API to send a message to a channel, and all subscribers will receive it.

### Example

```java
Cloud.channels().publishString("testChannel", "MESSAGE");
```

In this example, the string `"MESSAGE"` is broadcast to all clients subscribed to the "testChannel" channel.