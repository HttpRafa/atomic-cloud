package io.atomic.cloud.common.connection;

import com.google.common.util.concurrent.FutureCallback;
import com.google.common.util.concurrent.Futures;
import com.google.common.util.concurrent.ListenableFuture;
import com.google.protobuf.BoolValue;
import com.google.protobuf.Empty;
import com.google.protobuf.StringValue;
import com.google.protobuf.UInt32Value;
import io.atomic.cloud.common.cache.CachedObject;
import io.atomic.cloud.common.connection.call.CallHandle;
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
import java.util.concurrent.Executors;
import lombok.RequiredArgsConstructor;
import org.jetbrains.annotations.Contract;
import org.jetbrains.annotations.NotNull;

@SuppressWarnings("UnusedReturnValue")
@RequiredArgsConstructor
public class CloudConnection {

    private static final Executor EXECUTOR = Executors.newCachedThreadPool();

    private final URL address;
    private final String token;

    private ClientServiceGrpc.ClientServiceStub client;
    private ClientServiceGrpc.ClientServiceFutureStub futureClient;

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
        var credentials = new CallCredentials() {
            @Override
            public void applyRequestMetadata(
                    RequestInfo requestInfo, Executor executor, @NotNull MetadataApplier applier) {
                var metadata = new Metadata();
                metadata.put(Metadata.Key.of("authorization", Metadata.ASCII_STRING_MARSHALLER), token);
                applier.apply(metadata);
            }
        };

        var managedChannel = channel.build();
        this.client = ClientServiceGrpc.newStub(managedChannel).withCallCredentials(credentials);
        this.futureClient = ClientServiceGrpc.newFutureStub(managedChannel).withCallCredentials(credentials);
    }

    public CompletableFuture<Empty> beat() {
        return this.wrapInFuture(this.futureClient.beat(Empty.getDefaultInstance()));
    }

    public CompletableFuture<Empty> setReady(boolean ready) {
        return this.wrapInFuture(this.futureClient.setReady(BoolValue.of(ready)));
    }

    public CompletableFuture<Empty> setRunning() {
        return this.wrapInFuture(this.futureClient.setRunning(Empty.getDefaultInstance()));
    }

    public CompletableFuture<Empty> requestStop() {
        return this.wrapInFuture(this.futureClient.requestStop(Empty.getDefaultInstance()));
    }

    public CompletableFuture<Empty> userConnected(User.ConnectedReq user) {
        return this.wrapInFuture(this.futureClient.userConnected(user));
    }

    public CompletableFuture<Empty> userDisconnected(User.DisconnectedReq user) {
        return this.wrapInFuture(this.futureClient.userDisconnected(user));
    }

    public CompletableFuture<UInt32Value> transferUsers(Transfer.TransferReq transfer) {
        return this.wrapInFuture(this.futureClient.transferUsers(transfer));
    }

    public CompletableFuture<UInt32Value> publishMessage(Channel.Msg message) {
        return this.wrapInFuture(this.futureClient.publishMessage(message));
    }

    public synchronized Optional<Server.List> getServersNow() {
        var cached = this.serversInfo.getValue();
        if (cached.isEmpty()) {
            this.getServers(); // Request value from controller
        }
        return cached;
    }

    public synchronized CompletableFuture<Server.List> getServers() {
        return this.serversInfo
                .getValue()
                .map(CompletableFuture::completedFuture)
                .orElseGet(() -> this.wrapInFuture(this.futureClient.getServers(Empty.getDefaultInstance()))
                        .thenApply((value) -> {
                            this.serversInfo.setValue(value);
                            return value;
                        }));
    }

    public synchronized Optional<Group.List> getGroupsNow() {
        var cached = this.groupsInfo.getValue();
        if (cached.isEmpty()) {
            this.getGroups(); // Request value from controller
        }
        return cached;
    }

    public synchronized CompletableFuture<Group.List> getGroups() {
        return this.groupsInfo
                .getValue()
                .map(CompletableFuture::completedFuture)
                .orElseGet(() -> this.wrapInFuture(this.futureClient.getGroups(Empty.getDefaultInstance()))
                        .thenApply((value) -> {
                            this.groupsInfo.setValue(value);
                            return value;
                        }));
    }

    public synchronized CompletableFuture<UInt32Value> getProtoVer() {
        return this.protocolVersion
                .getValue()
                .map(CompletableFuture::completedFuture)
                .orElseGet(() -> this.wrapInFuture(this.futureClient.getProtoVer(Empty.getDefaultInstance()))
                        .thenApply((value) -> {
                            this.protocolVersion.setValue(value);
                            return value;
                        }));
    }

    public synchronized CompletableFuture<StringValue> getCtrlVer() {
        return this.controllerVersion
                .getValue()
                .map(CompletableFuture::completedFuture)
                .orElseGet(() -> this.wrapInFuture(this.futureClient.getCtrlVer(Empty.getDefaultInstance()))
                        .thenApply((value) -> {
                            this.controllerVersion.setValue(value);
                            return value;
                        }));
    }

    /* Subscriptions */
    public CallHandle<?, Transfer.TransferRes> subscribeToTransfers(StreamObserver<Transfer.TransferRes> observer) {
        var handle = new CallHandle<>(observer);
        this.client.subscribeToTransfers(Empty.getDefaultInstance(), handle);
        return handle;
    }

    public CallHandle<?, Channel.Msg> subscribeToChannel(String channel, StreamObserver<Channel.Msg> observer) {
        var handle = new CallHandle<>(observer);
        this.client.subscribeToChannel(StringValue.of(channel), handle);
        return handle;
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

    private <T> @NotNull CompletableFuture<T> wrapInFuture(@NotNull ListenableFuture<T> future) {
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
