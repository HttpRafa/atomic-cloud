Channels

# What are Channels and What Are They Used For?

Channels are a pub/sub (publish/subscribe) messaging system, similar to protocols like MQTT, designed to enable efficient communication between different servers. By leveraging channels, you can implement secure, real-time message passing across servers, which is particularly useful for features such as party systems, player notifications, or any other functionality that requires inter-server communication.

Channels provide a central, consistent method for servers to send and receive messages, ensuring data integrity and security across distributed environments. This makes them a powerful tool for orchestrating communication between different services or components within a distributed application.

# Prerequisites for Using Channels

To effectively use channels, you must select a unique channel name. This name is used to identify the communication channel between servers. Once a channel is established, any server or client that subscribes to that specific channel will receive messages sent to it. They will continue to receive messages until they explicitly unsubscribe.

## Key Considerations:
Channel Naming: Channel names are unique identifiers that determine the flow of messages.
Subscribing: Servers or clients must subscribe to a channel to receive messages.
Unsubscribing: Subscribers can unsubscribe from a channel at any time to stop receiving messages.
Receiving Messages

To receive messages from a channel, you must first subscribe to the channel via the JVM API. This process is simple and can be easily integrated into your controller.

### Important Note:
For message subscription to work, a connection setup is required. In this example, we use the Paper client as the connection interface, but this setup can be adapted for any environment where the Cloud class is properly configured.

#### Example:
```JAVA
// Subscribe to the "party/invites" channel
Cloud.channels().subscribe("party/invites");

// Register a handler to process messages received on the "party/invites" channel
Cloud.channels().registerHandler("party/invites", (message) -> {
    // TODO: Define your message handling logic here
});
```
In this example, once subscribed, your handler will be called every time a message is received on the party/invites channel. You can define the handler logic to process the incoming messages as needed.

### Sending Messages

Sending messages through channels is straightforward. You can send a message to any channel to notify subscribers or trigger specific actions.

#### Example:
```JAVA
// Send a message to the "party/invites" channel
Cloud.channels().sendMessage("party/invites", "test");
```
In this example, the message "test" will be sent to all subscribers of the party/invites channel.

### Conclusion

Channels provide a robust and secure mechanism for communication between servers, enabling real-time interactions across your application. By subscribing to and sending messages through channels, you can implement complex inter-server communication patterns with minimal setup.