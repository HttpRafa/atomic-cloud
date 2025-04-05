package io.atomic.cloud.common.connection;

import com.google.common.util.concurrent.FutureCallback;
import com.google.common.util.concurrent.Futures;
import com.google.common.util.concurrent.ListenableFuture;
import io.atomic.cloud.grpc.client.*;
import io.grpc.*;
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Executor;
import java.util.concurrent.Executors;
import lombok.Getter;
import lombok.RequiredArgsConstructor;
import org.jetbrains.annotations.NotNull;

@SuppressWarnings("UnusedReturnValue")
@RequiredArgsConstructor
@Getter
public abstract class Connection {

    protected static final Executor EXECUTOR = Executors.newCachedThreadPool();

    protected final URL address;
    protected final String token;
    protected final String certificate;

    public abstract void connect() throws IOException;

    protected ManagedChannel createChannel() throws IOException {
        ManagedChannelBuilder<?> channel;
        if (this.certificate != null) {
            channel = Grpc.newChannelBuilderForAddress(
                    this.address.getHost(),
                    this.address.getPort(),
                    TlsChannelCredentials.newBuilder()
                            .trustManager(new ByteArrayInputStream(this.certificate.getBytes(StandardCharsets.UTF_8)))
                            .build());
        } else {
            channel = ManagedChannelBuilder.forAddress(this.address.getHost(), this.address.getPort());
            channel.usePlaintext();
        }

        return channel.build();
    }

    protected <T> @NotNull CompletableFuture<T> wrapInFuture(@NotNull ListenableFuture<T> future) {
        var newFuture = new CompletableFuture<T>();
        Futures.addCallback(
                future,
                new FutureCallback<>() {
                    @Override
                    public void onSuccess(T result) {
                        newFuture.complete(result);
                    }

                    @Override
                    public void onFailure(@NotNull Throwable throwable) {
                        newFuture.completeExceptionally(throwable);
                    }
                },
                EXECUTOR);
        return newFuture;
    }
}
