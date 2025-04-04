package io.atomic.cloud.common.channel;

import com.google.protobuf.ByteString;
import com.google.protobuf.UInt32Value;
import io.atomic.cloud.api.channel.Channels;
import io.atomic.cloud.api.channel.subscription.Bytes;
import io.atomic.cloud.common.channel.subscription.BytesImpl;
import io.atomic.cloud.common.connection.client.ClientConnection;
import io.atomic.cloud.grpc.client.Channel;
import java.util.concurrent.CompletableFuture;
import lombok.AllArgsConstructor;

@AllArgsConstructor
public class ChannelManager implements Channels {

    private final ClientConnection connection;

    @Override
    public CompletableFuture<Integer> publishBytes(String channel, byte[] data) {
        return this.connection
                .publishMessage(Channel.Msg.newBuilder()
                        .setChannel(channel)
                        .setData(ByteString.copyFrom(data))
                        .setTimestamp(System.currentTimeMillis())
                        .build())
                .thenApply(UInt32Value::getValue);
    }

    @Override
    public Bytes subscribeToBytes(String channel) {
        return BytesImpl.create(channel, this.connection);
    }
}
