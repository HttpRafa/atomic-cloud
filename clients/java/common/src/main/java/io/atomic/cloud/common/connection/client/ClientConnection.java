package io.atomic.cloud.common.connection.client;

import com.google.protobuf.BoolValue;
import com.google.protobuf.Empty;
import com.google.protobuf.StringValue;
import com.google.protobuf.UInt32Value;
import io.atomic.cloud.common.cache.CachedObject;
import io.atomic.cloud.common.connection.Connection;
import io.atomic.cloud.common.connection.call.CallHandle;
import io.atomic.cloud.common.connection.credential.TokenCredential;
import io.atomic.cloud.grpc.client.*;
import io.atomic.cloud.grpc.common.Notify;
import io.grpc.stub.StreamObserver;
import java.io.IOException;
import java.net.MalformedURLException;
import java.net.URI;
import java.net.URISyntaxException;
import java.net.URL;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import org.jetbrains.annotations.Contract;
import org.jetbrains.annotations.NotNull;

public class ClientConnection extends Connection {

    private ClientServiceGrpc.ClientServiceStub client;
    private ClientServiceGrpc.ClientServiceFutureStub futureClient;

    // Cache values
    private final CachedObject<UInt32Value> userCount = new CachedObject<>();
    private final CachedObject<UInt32Value> protocolVersion = new CachedObject<>();
    private final CachedObject<StringValue> controllerVersion = new CachedObject<>();
    private final CachedObject<Server.List> serversInfo = new CachedObject<>();
    private final CachedObject<Group.List> groupsInfo = new CachedObject<>();

    public ClientConnection(URL address, String token, String certificate) {
        super(address, token, certificate);
    }

    @Override
    public void connect() throws IOException {
        var credentials = new TokenCredential(super.token);
        var managedChannel = super.createChannel();
        this.client = ClientServiceGrpc.newStub(managedChannel).withCallCredentials(credentials);
        this.futureClient = ClientServiceGrpc.newFutureStub(managedChannel).withCallCredentials(credentials);
    }

    public CompletableFuture<Empty> beat() {
        return super.wrapInFuture(this.futureClient.beat(Empty.getDefaultInstance()));
    }

    public CompletableFuture<Empty> ready(boolean ready) {
        return super.wrapInFuture(this.futureClient.setReady(BoolValue.of(ready)));
    }

    public CompletableFuture<Empty> running() {
        return super.wrapInFuture(this.futureClient.setRunning(Empty.getDefaultInstance()));
    }

    public CompletableFuture<Empty> requestStop() {
        return super.wrapInFuture(this.futureClient.requestStop(Empty.getDefaultInstance()));
    }

    public CompletableFuture<Empty> userConnected(User.ConnectedReq user) {
        return super.wrapInFuture(this.futureClient.userConnected(user));
    }

    public CompletableFuture<Empty> userDisconnected(User.DisconnectedReq user) {
        return super.wrapInFuture(this.futureClient.userDisconnected(user));
    }

    public CompletableFuture<UInt32Value> transferUsers(Transfer.TransferReq transfer) {
        return super.wrapInFuture(this.futureClient.transferUsers(transfer));
    }

    public CompletableFuture<UInt32Value> publishMessage(Channel.Msg message) {
        return super.wrapInFuture(this.futureClient.publishMessage(message));
    }

    public synchronized Optional<UInt32Value> userCountNow() {
        var cached = this.userCount.getValue();
        if (cached.isEmpty()) {
            this.userCount(); // Request value from controller
        }
        return cached;
    }

    public synchronized CompletableFuture<UInt32Value> userCount() {
        return this.userCount.getValue().map(CompletableFuture::completedFuture).orElseGet(() -> super.wrapInFuture(
                        this.futureClient.getUserCount(Empty.getDefaultInstance()))
                .thenApply((value) -> {
                    this.userCount.setValue(value);
                    return value;
                }));
    }

    public synchronized Optional<Server.List> serversNow() {
        var cached = this.serversInfo.getValue();
        if (cached.isEmpty()) {
            this.servers(); // Request value from controller
        }
        return cached;
    }

    public synchronized CompletableFuture<Server.List> servers() {
        return this.serversInfo
                .getValue()
                .map(CompletableFuture::completedFuture)
                .orElseGet(() -> super.wrapInFuture(this.futureClient.getServers(Empty.getDefaultInstance()))
                        .thenApply((value) -> {
                            this.serversInfo.setValue(value);
                            return value;
                        }));
    }

    public synchronized Optional<Group.List> groupsNow() {
        var cached = this.groupsInfo.getValue();
        if (cached.isEmpty()) {
            this.groups(); // Request value from controller
        }
        return cached;
    }

    public synchronized CompletableFuture<Group.List> groups() {
        return this.groupsInfo
                .getValue()
                .map(CompletableFuture::completedFuture)
                .orElseGet(() -> super.wrapInFuture(this.futureClient.getGroups(Empty.getDefaultInstance()))
                        .thenApply((value) -> {
                            this.groupsInfo.setValue(value);
                            return value;
                        }));
    }

    public synchronized CompletableFuture<UInt32Value> protoVer() {
        return this.protocolVersion
                .getValue()
                .map(CompletableFuture::completedFuture)
                .orElseGet(() -> super.wrapInFuture(this.futureClient.getProtoVer(Empty.getDefaultInstance()))
                        .thenApply((value) -> {
                            this.protocolVersion.setValue(value);
                            return value;
                        }));
    }

    public synchronized CompletableFuture<StringValue> ctrlVer() {
        return this.controllerVersion
                .getValue()
                .map(CompletableFuture::completedFuture)
                .orElseGet(() -> super.wrapInFuture(this.futureClient.getCtrlVer(Empty.getDefaultInstance()))
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

    /* Notify */
    public CallHandle<?, Notify.PowerEvent> subscribeToPowerEvents(StreamObserver<Notify.PowerEvent> observer) {
        var handle = new CallHandle<>(observer);
        this.client.subscribeToPowerEvents(Empty.getDefaultInstance(), handle);
        return handle;
    }

    public CallHandle<?, Notify.ReadyEvent> subscribeToReadyEvents(StreamObserver<Notify.ReadyEvent> observer) {
        var handle = new CallHandle<>(observer);
        this.client.subscribeToReadyEvents(Empty.getDefaultInstance(), handle);
        return handle;
    }

    @Contract(" -> new")
    public static @NotNull ClientConnection createFromEnv() {
        var address = System.getenv("CONTROLLER_ADDRESS");
        var token = System.getenv("SERVER_TOKEN");
        if (address == null) {
            throw new IllegalStateException("CONTROLLER_ADDRESS not set");
        } else if (token == null) {
            throw new IllegalStateException("SERVER_TOKEN not set");
        }

        var certificate = System.getenv("CONTROLLER_CERTIFICATE");

        URL url;
        try {
            url = new URI(address).toURL();
        } catch (MalformedURLException | URISyntaxException exception) {
            throw new IllegalStateException("Failed to parse CONTROLLER_ADDRESS variable", exception);
        }
        return new ClientConnection(url, token, certificate);
    }
}
