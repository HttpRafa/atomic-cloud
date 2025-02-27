package io.atomic.cloud.common.channel;

import com.google.protobuf.ByteString;
import com.google.protobuf.UInt32Value;
import io.atomic.cloud.api.channel.Channels;
import io.atomic.cloud.api.channel.subscription.Bytes;
import io.atomic.cloud.common.connection.CloudConnection;
import java.util.concurrent.CompletableFuture;

import io.atomic.cloud.grpc.client.Channel;
import lombok.AllArgsConstructor;

@AllArgsConstructor
public class ChannelManager implements Channels {

    private final CloudConnection connection;

    @Override
    public CompletableFuture<Integer> publishBytes(String channel, byte[] data) {
        return connection.publishMessage(Channel.Msg.newBuilder().setChannel(channel).setData(ByteString.copyFrom(data)).setTimestamp(System.currentTimeMillis()).build()).thenApply(UInt32Value::getValue);
    }

    @Override
    public Bytes subscribeToBytes(String channel) {
        return null;
    }

    public void cleanup() {

    }
}
