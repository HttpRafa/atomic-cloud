package io.atomic.cloud.common.connection.client;

import com.google.protobuf.Empty;
import com.google.protobuf.StringValue;
import com.google.protobuf.UInt32Value;
import io.atomic.cloud.common.cache.CachedObject;
import io.atomic.cloud.common.connection.Connection;
import io.atomic.cloud.common.connection.call.CallHandle;
import io.atomic.cloud.common.connection.credential.TokenCredential;
import io.atomic.cloud.grpc.common.Notify;
import io.atomic.cloud.grpc.manage.*;
import io.grpc.stub.StreamObserver;
import java.io.IOException;
import java.net.URL;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import org.jetbrains.annotations.Contract;
import org.jetbrains.annotations.NotNull;

public class ManageConnection extends Connection {

    private ManageServiceGrpc.ManageServiceStub client;
    private ManageServiceGrpc.ManageServiceFutureStub futureClient;

    // Cache values
    private final CachedObject<UInt32Value> protocolVersion = new CachedObject<>();
    private final CachedObject<StringValue> controllerVersion = new CachedObject<>();
    private final CachedObject<Plugin.List> pluginsInfo = new CachedObject<>();
    private final CachedObject<Node.List> nodesInfo = new CachedObject<>();
    private final CachedObject<Group.List> groupsInfo = new CachedObject<>();
    private final CachedObject<Server.List> serversInfo = new CachedObject<>();
    private final CachedObject<User.List> usersInfo = new CachedObject<>();

    public ManageConnection(URL address, String token, String certificate) {
        super(address, token, certificate);
    }

    @Override
    public void connect() throws IOException {
        var credentials = new TokenCredential(super.token);
        var managedChannel = super.createChannel();
        this.client = ManageServiceGrpc.newStub(managedChannel).withCallCredentials(credentials);
        this.futureClient = ManageServiceGrpc.newFutureStub(managedChannel).withCallCredentials(credentials);
    }

    public CompletableFuture<Empty> requestStop() {
        return super.wrapInFuture(this.futureClient.requestStop(Empty.getDefaultInstance()));
    }

    public CompletableFuture<Empty> setResource(Resource.SetReq request) {
        return super.wrapInFuture(this.futureClient.setResource(request));
    }

    public CompletableFuture<Empty> deleteResource(Resource.DelReq request) {
        return super.wrapInFuture(this.futureClient.deleteResource(request));
    }

    public CompletableFuture<Empty> createNode(Node.Item node) {
        return super.wrapInFuture(this.futureClient.createNode(node));
    }

    public CompletableFuture<Empty> createGroup(Group.Item group) {
        return super.wrapInFuture(this.futureClient.createGroup(group));
    }

    public CompletableFuture<Empty> writeToScreen(Screen.WriteReq request) {
        return super.wrapInFuture(this.futureClient.writeToScreen(request));
    }

    public CompletableFuture<UInt32Value> transferUsers(Transfer.TransferReq request) {
        return super.wrapInFuture(this.futureClient.transferUsers(request));
    }

    public CompletableFuture<Node.Item> node(String node) {
        return super.wrapInFuture(this.futureClient.getNode(StringValue.of(node)));
    }

    public CompletableFuture<Group.Item> group(String group) {
        return super.wrapInFuture(this.futureClient.getGroup(StringValue.of(group)));
    }

    public CompletableFuture<Server.Detail> server(String server) {
        return super.wrapInFuture(this.futureClient.getServer(StringValue.of(server)));
    }

    public synchronized Optional<Plugin.List> pluginsNow() {
        var cached = this.pluginsInfo.getValue();
        if (cached.isEmpty()) {
            this.plugins(); // Request value from controller
        }
        return cached;
    }

    public synchronized CompletableFuture<Plugin.List> plugins() {
        return this.pluginsInfo
                .getValue()
                .map(CompletableFuture::completedFuture)
                .orElseGet(() -> super.wrapInFuture(this.futureClient.getPlugins(Empty.getDefaultInstance()))
                        .thenApply((value) -> {
                            this.pluginsInfo.setValue(value);
                            return value;
                        }));
    }

    public synchronized Optional<Node.List> nodesNow() {
        var cached = this.nodesInfo.getValue();
        if (cached.isEmpty()) {
            this.nodes(); // Request value from controller
        }
        return cached;
    }

    public synchronized CompletableFuture<Node.List> nodes() {
        return this.nodesInfo.getValue().map(CompletableFuture::completedFuture).orElseGet(() -> super.wrapInFuture(
                        this.futureClient.getNodes(Empty.getDefaultInstance()))
                .thenApply((value) -> {
                    this.nodesInfo.setValue(value);
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

    public synchronized Optional<User.List> usersNow() {
        var cached = this.usersInfo.getValue();
        if (cached.isEmpty()) {
            this.users(); // Request value from controller
        }
        return cached;
    }

    public synchronized CompletableFuture<User.List> users() {
        return this.usersInfo.getValue().map(CompletableFuture::completedFuture).orElseGet(() -> super.wrapInFuture(
                        this.futureClient.getUsers(Empty.getDefaultInstance()))
                .thenApply((value) -> {
                    this.usersInfo.setValue(value);
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

    /* Screen */
    public CallHandle<?, Screen.Lines> subscribeToScreen(String server, StreamObserver<Screen.Lines> observer) {
        var handle = new CallHandle<>(observer);
        this.client.subscribeToScreen(StringValue.of(server), handle);
        return handle;
    }

    /* Notify */
    public CallHandle<?, Notify.PowerEvent> subscribeToPowerEvents(StreamObserver<Notify.PowerEvent> observer) {
        var handle = new CallHandle<>(observer);
        this.client.subscribeToPowerEvents(Empty.getDefaultInstance(), handle);
        return handle;
    }

    @Contract("_, _ -> new")
    public static @NotNull ManageConnection createFromOther(@NotNull ClientConnection connection, String token) {
        return new ManageConnection(connection.address(), token, connection.certificate());
    }
}
