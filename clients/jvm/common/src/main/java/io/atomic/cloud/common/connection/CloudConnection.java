package io.atomic.cloud.common.connection;

import com.google.protobuf.BoolValue;
import com.google.protobuf.Empty;
import com.google.protobuf.StringValue;
import com.google.protobuf.UInt32Value;
import io.atomic.cloud.grpc.server.*;
import io.grpc.CallCredentials;
import io.grpc.ManagedChannelBuilder;
import io.grpc.Metadata;
import io.grpc.stub.StreamObserver;
import java.net.MalformedURLException;
import java.net.URI;
import java.net.URISyntaxException;
import java.net.URL;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Executor;
import lombok.Getter;
import lombok.RequiredArgsConstructor;
import org.jetbrains.annotations.Contract;
import org.jetbrains.annotations.NotNull;

@RequiredArgsConstructor
public class CloudConnection {

    private final URL address;
    private final String token;

    private ServerServiceGrpc.ServerServiceStub client;

    // Cache values
    private String controllerVersion;

    public void connect() {
        var channel = ManagedChannelBuilder.forAddress(this.address.getHost(), this.address.getPort());
        if (this.address.getProtocol().equals("https")) {
            channel.useTransportSecurity();
        } else {
            channel.usePlaintext();
        }

        this.client = ServerServiceGrpc.newStub(channel.build()).withCallCredentials(new CallCredentials() {
            @Override
            public void applyRequestMetadata(RequestInfo requestInfo, Executor executor, MetadataApplier applier) {
                var metadata = new Metadata();
                metadata.put(Metadata.Key.of("authorization", Metadata.ASCII_STRING_MARSHALLER), token);
                applier.apply(metadata);
            }
        });
    }

    public CompletableFuture<Empty> beatHeart() {
        var observer = new StreamObserverImpl<Empty>();
        this.client.beatHeart(Empty.getDefaultInstance(), observer);
        return observer.future();
    }

    public CompletableFuture<Empty> markReady() {
        var observer = new StreamObserverImpl<Empty>();
        this.client.markReady(Empty.getDefaultInstance(), observer);
        return observer.future();
    }

    public CompletableFuture<Empty> markNotReady() {
        var observer = new StreamObserverImpl<Empty>();
        this.client.markNotReady(Empty.getDefaultInstance(), observer);
        return observer.future();
    }

    public CompletableFuture<Empty> markRunning() {
        var observer = new StreamObserverImpl<Empty>();
        this.client.markRunning(Empty.getDefaultInstance(), observer);
        return observer.future();
    }

    public CompletableFuture<Empty> requestStop() {
        var observer = new StreamObserverImpl<Empty>();
        this.client.requestStop(Empty.getDefaultInstance(), observer);
        return observer.future();
    }

    public CompletableFuture<Empty> userConnected(UserConnectedRequest user) {
        var observer = new StreamObserverImpl<Empty>();
        this.client.userConnected(user, observer);
        return observer.future();
    }

    public CompletableFuture<Empty> userDisconnected(UserDisconnectedRequest user) {
        var observer = new StreamObserverImpl<Empty>();
        this.client.userDisconnected(user, observer);
        return observer.future();
    }

    public void subscribeToTransfers(StreamObserver<ResolvedTransferResponse> observer) {
        this.client.subscribeToTransfers(Empty.getDefaultInstance(), observer);
    }

    public CompletableFuture<UInt32Value> transferAllUsers(TransferAllUsersRequest target) {
        var observer = new StreamObserverImpl<UInt32Value>();
        this.client.transferAllUsers(target, observer);
        return observer.future();
    }

    public CompletableFuture<BoolValue> transferUser(TransferUserRequest transfer) {
        var observer = new StreamObserverImpl<BoolValue>();
        this.client.transferUser(transfer, observer);
        return observer.future();
    }

    public CompletableFuture<UInt32Value> sendMessageToChannel(ChannelMessageValue message) {
        var observer = new StreamObserverImpl<UInt32Value>();
        this.client.sendMessageToChannel(message, observer);
        return observer.future();
    }

    public CompletableFuture<Empty> unsubscribeFromChannel(String channel) {
        var observer = new StreamObserverImpl<Empty>();
        this.client.unsubscribeFromChannel(StringValue.of(channel), observer);
        return observer.future();
    }

    public void subscribeToChannel(String channel, StreamObserver<ChannelMessageValue> observer) {
        this.client.subscribeToChannel(StringValue.of(channel), observer);
    }

    public CompletableFuture<Empty> sendReset() {
        var observer = new StreamObserverImpl<Empty>();
        this.client.reset(Empty.getDefaultInstance(), observer);
        return observer.future();
    }

    public CompletableFuture<StringValue> getControllerVersion() {
        if (this.controllerVersion != null) {
            return CompletableFuture.completedFuture(StringValue.of(this.controllerVersion));
        }
        var observer = new StreamObserverImpl<StringValue>();
        this.client.getControllerVersion(Empty.getDefaultInstance(), observer);
        return observer.future();
    }

    @Contract(" -> new")
    public static @NotNull CloudConnection createFromEnv() {
        var address = System.getenv("CONTROLLER_ADDRESS");
        var token = System.getenv("SERVER_TOKEN");
        if (address == null) {
            throw new IllegalStateException("CONTROLLER_ADDRESS not set");
        } else if (token == null) {
            throw new IllegalStateException("SERVER_TOKEN not set");
        }

        URL url;
        try {
            url = new URI(address).toURL();
        } catch (MalformedURLException | URISyntaxException exception) {
            throw new IllegalStateException("Failed to parse CONTROLLER_ADDRESS variable", exception);
        }
        return new CloudConnection(url, token);
    }

    @Getter
    public static class StreamObserverImpl<T> implements StreamObserver<T> {

        private final CompletableFuture<T> future = new CompletableFuture<>();

        @Override
        public void onNext(T value) {
            future.complete(value);
        }

        @Override
        public void onError(Throwable t) {
            future.completeExceptionally(t);
        }

        @Override
        public void onCompleted() {}
    }
}
