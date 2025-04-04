package io.atomic.cloud.common.channel.subscription;

import io.atomic.cloud.api.channel.message.ByteMessage;
import io.atomic.cloud.api.channel.subscription.Bytes;
import io.atomic.cloud.common.connection.call.CallHandle;
import io.atomic.cloud.common.connection.client.ClientConnection;
import io.atomic.cloud.grpc.client.Channel;
import io.grpc.stub.StreamObserver;
import java.util.function.Consumer;
import org.jetbrains.annotations.Contract;
import org.jetbrains.annotations.NotNull;

public class BytesImpl implements StreamObserver<Channel.Msg>, Bytes {

    private Consumer<ByteMessage> handler;
    private Consumer<Throwable> errorHandler;
    private CallHandle<?, ?> handle;

    @Contract("_, _ -> new")
    public static @NotNull BytesImpl create(String channel, @NotNull ClientConnection connection) {
        var impl = new BytesImpl();
        impl.handle = connection.subscribeToChannel(channel, impl);
        return impl;
    }

    @Override
    public void close() {
        this.handle.cancel();
    }

    @Override
    public void onNext(Channel.Msg value) {
        if (this.handler != null) {
            handler.accept(new ByteMessage(value.getTimestamp(), value.getData().toByteArray()));
        }
    }

    @Override
    public void onError(Throwable throwable) {
        if (this.errorHandler != null) {
            this.errorHandler.accept(throwable);
        }
    }

    @Override
    public void onCompleted() {
        // Do nothing
    }

    @Override
    public void handler(Consumer<ByteMessage> handler) {
        this.handler = handler;
    }

    @Override
    public void errorHandler(Consumer<Throwable> handler) {
        this.errorHandler = handler;
    }
}
