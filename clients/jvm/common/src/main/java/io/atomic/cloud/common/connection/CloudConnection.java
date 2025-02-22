package io.atomic.cloud.common.connection;

import com.google.protobuf.BoolValue;
import com.google.protobuf.Empty;
import com.google.protobuf.StringValue;
import com.google.protobuf.UInt32Value;
import io.atomic.cloud.common.cache.CachedObject;
import io.atomic.cloud.grpc.client.*;
import io.grpc.CallCredentials;
import io.grpc.ManagedChannelBuilder;
import io.grpc.Metadata;
import io.grpc.stub.StreamObserver;
import java.net.MalformedURLException;
import java.net.URI;
import java.net.URISyntaxException;
import java.net.URL;
import java.util.Optional;
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

    private ClientServiceGrpc.ClientServiceStub client;

    // Cache values
    private final CachedObject<UInt32Value> protocolVersion = new CachedObject<>();
    private final CachedObject<StringValue> controllerVersion = new CachedObject<>();
    private final CachedObject<Server.List> serversInfo = new CachedObject<>();
    private final CachedObject<Group.List> groupsInfo = new CachedObject<>();

    public void connect() {
        var channel = ManagedChannelBuilder.forAddress(this.address.getHost(), this.address.getPort());
        if (this.address.getProtocol().equals("https")) {
            channel.useTransportSecurity();
        } else {
            channel.usePlaintext();
        }

        this.client = ClientServiceGrpc.newStub(channel.build()).withCallCredentials(new CallCredentials() {
            @Override
            public void applyRequestMetadata(RequestInfo requestInfo, Executor executor, MetadataApplier applier) {
                var metadata = new Metadata();
                metadata.put(Metadata.Key.of("authorization", Metadata.ASCII_STRING_MARSHALLER), token);
                applier.apply(metadata);
            }
        });
    }

    public CompletableFuture<Empty> beat() {
        var observer = new StreamObserverImpl<Empty>();
        this.client.beat(Empty.getDefaultInstance(), observer);
        return observer.future();
    }

    public CompletableFuture<Empty> setReady(boolean ready) {
        var observer = new StreamObserverImpl<Empty>();
        this.client.setReady(BoolValue.of(ready), observer);
        return observer.future();
    }

    public CompletableFuture<Empty> setRunning() {
        var observer = new StreamObserverImpl<Empty>();
        this.client.setRunning(Empty.getDefaultInstance(), observer);
        return observer.future();
    }

    public CompletableFuture<Empty> requestStop() {
        var observer = new StreamObserverImpl<Empty>();
        this.client.requestStop(Empty.getDefaultInstance(), observer);
        return observer.future();
    }

    public CompletableFuture<Empty> userConnected(User.ConnectedReq user) {
        var observer = new StreamObserverImpl<Empty>();
        this.client.userConnected(user, observer);
        return observer.future();
    }

    public CompletableFuture<Empty> userDisconnected(User.DisconnectedReq user) {
        var observer = new StreamObserverImpl<Empty>();
        this.client.userDisconnected(user, observer);
        return observer.future();
    }

    public void subscribeToTransfers(StreamObserver<Transfer.TransferRes> observer) {
        this.client.subscribeToTransfers(Empty.getDefaultInstance(), observer);
    }

    public CompletableFuture<UInt32Value> transferUsers(Transfer.TransferReq transfer) {
        var observer = new StreamObserverImpl<UInt32Value>();
        this.client.transferUsers(transfer, observer);
        return observer.future();
    }

    public CompletableFuture<UInt32Value> publishMessage(Channel.Msg message) {
        var observer = new StreamObserverImpl<UInt32Value>();
        this.client.publishMessage(message, observer);
        return observer.future();
    }

    public void subscribeToChannel(String channel, StreamObserver<Channel.Msg> observer) {
        this.client.subscribeToChannel(StringValue.of(channel), observer);
    }

    public synchronized Optional<Server.List> getServersNow() {
        var cached = this.serversInfo.getValue();
        if (cached.isEmpty()) {
            this.getServers(); // Request value from controller
        }
        return cached;
    }

    public synchronized CompletableFuture<Server.List> getServers() {
        var cached = this.serversInfo.getValue();
        if (cached.isPresent()) {
            return CompletableFuture.completedFuture(cached.get());
        }
        var observer = new StreamObserverImpl<Server.List>();
        this.client.getServers(Empty.getDefaultInstance(), observer);
        return observer.future().thenApply((value) -> {
            this.serversInfo.setValue(value);
            return value;
        });
    }

    public synchronized Optional<Group.List> getGroupsNow() {
        var cached = this.groupsInfo.getValue();
        if (cached.isEmpty()) {
            this.getGroups(); // Request value from controller
        }
        return cached;
    }

    public synchronized CompletableFuture<Group.List> getGroups() {
        var cached = this.groupsInfo.getValue();
        if (cached.isPresent()) {
            return CompletableFuture.completedFuture(cached.get());
        }
        var observer = new StreamObserverImpl<Group.List>();
        this.client.getGroups(Empty.getDefaultInstance(), observer);
        return observer.future().thenApply((value) -> {
            this.groupsInfo.setValue(value);
            return value;
        });
    }

    public synchronized CompletableFuture<UInt32Value> getProtoVer() {
        var cached = this.protocolVersion.getValue();
        if (cached.isPresent()) {
            return CompletableFuture.completedFuture(cached.get());
        }
        var observer = new StreamObserverImpl<UInt32Value>();
        this.client.getProtoVer(Empty.getDefaultInstance(), observer);
        return observer.future().thenApply((value) -> {
            this.protocolVersion.setValue(value);
            return value;
        });
    }

    public synchronized CompletableFuture<StringValue> getCtrlVer() {
        var cached = this.controllerVersion.getValue();
        if (cached.isPresent()) {
            return CompletableFuture.completedFuture(cached.get());
        }
        var observer = new StreamObserverImpl<StringValue>();
        this.client.getCtrlVer(Empty.getDefaultInstance(), observer);
        return observer.future().thenApply((value) -> {
            this.controllerVersion.setValue(value);
            return value;
        });
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
