package io.atomic.cloud.common.connection;

import com.google.protobuf.Empty;
import com.google.protobuf.StringValue;
import com.google.protobuf.UInt32Value;
import io.atomic.cloud.common.cache.CachedObject;
import io.atomic.cloud.grpc.server.*;
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

    private UnitServiceGrpc.UnitServiceStub client;

    // Cache values
    private final CachedObject<UInt32Value> protocolVersion = new CachedObject<>();
    private final CachedObject<StringValue> controllerVersion = new CachedObject<>();
    private final CachedObject<UnitInformation.UnitListResponse> serversInfo = new CachedObject<>();
    private final CachedObject<DeploymentInformation.DeploymentListResponse> groupsInfo = new CachedObject<>();

    public void connect() {
        var channel = ManagedChannelBuilder.forAddress(this.address.getHost(), this.address.getPort());
        if (this.address.getProtocol().equals("https")) {
            channel.useTransportSecurity();
        } else {
            channel.usePlaintext();
        }

        this.client = UnitServiceGrpc.newStub(channel.build()).withCallCredentials(new CallCredentials() {
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

    public CompletableFuture<Empty> userConnected(UserManagement.UserConnectedRequest user) {
        var observer = new StreamObserverImpl<Empty>();
        this.client.userConnected(user, observer);
        return observer.future();
    }

    public CompletableFuture<Empty> userDisconnected(UserManagement.UserDisconnectedRequest user) {
        var observer = new StreamObserverImpl<Empty>();
        this.client.userDisconnected(user, observer);
        return observer.future();
    }

    public void subscribeToTransfers(StreamObserver<TransferManagement.ResolvedTransferResponse> observer) {
        this.client.subscribeToTransfers(Empty.getDefaultInstance(), observer);
    }

    public CompletableFuture<UInt32Value> transferUsers(TransferManagement.TransferUsersRequest transfer) {
        var observer = new StreamObserverImpl<UInt32Value>();
        this.client.transferUsers(transfer, observer);
        return observer.future();
    }

    public CompletableFuture<UInt32Value> sendMessageToChannel(ChannelManagement.ChannelMessageValue message) {
        var observer = new StreamObserverImpl<UInt32Value>();
        this.client.sendMessageToChannel(message, observer);
        return observer.future();
    }

    public CompletableFuture<Empty> unsubscribeFromChannel(String channel) {
        var observer = new StreamObserverImpl<Empty>();
        this.client.unsubscribeFromChannel(StringValue.of(channel), observer);
        return observer.future();
    }

    public void subscribeToChannel(String channel, StreamObserver<ChannelManagement.ChannelMessageValue> observer) {
        this.client.subscribeToChannel(StringValue.of(channel), observer);
    }

    public Optional<UnitInformation.UnitListResponse> getUnitsNow() {
        var cached = this.serversInfo.getValue();
        if (cached.isEmpty()) {
            this.getUnits(); // Request value from controller
        }
        return cached;
    }

    public CompletableFuture<UnitInformation.UnitListResponse> getUnits() {
        var cached = this.serversInfo.getValue();
        if (cached.isPresent()) {
            return CompletableFuture.completedFuture(cached.get());
        }
        var observer = new StreamObserverImpl<UnitInformation.UnitListResponse>();
        this.client.getUnits(Empty.getDefaultInstance(), observer);
        return observer.future().thenApply((value) -> {
            this.serversInfo.setValue(value);
            return value;
        });
    }

    public Optional<DeploymentInformation.DeploymentListResponse> getDeploymentsNow() {
        var cached = this.groupsInfo.getValue();
        if (cached.isEmpty()) {
            this.getDeployments(); // Request value from controller
        }
        return cached;
    }

    public CompletableFuture<DeploymentInformation.DeploymentListResponse> getDeployments() {
        var cached = this.groupsInfo.getValue();
        if (cached.isPresent()) {
            return CompletableFuture.completedFuture(cached.get());
        }
        var observer = new StreamObserverImpl<DeploymentInformation.DeploymentListResponse>();
        this.client.getDeployments(Empty.getDefaultInstance(), observer);
        return observer.future().thenApply((value) -> {
            this.groupsInfo.setValue(value);
            return value;
        });
    }

    public CompletableFuture<Empty> sendReset() {
        var observer = new StreamObserverImpl<Empty>();
        this.client.reset(Empty.getDefaultInstance(), observer);
        return observer.future();
    }

    public synchronized CompletableFuture<UInt32Value> getProtocolVersion() {
        var cached = this.protocolVersion.getValue();
        if (cached.isPresent()) {
            return CompletableFuture.completedFuture(cached.get());
        }
        var observer = new StreamObserverImpl<UInt32Value>();
        this.client.getProtocolVersion(Empty.getDefaultInstance(), observer);
        return observer.future().thenApply((value) -> {
            this.protocolVersion.setValue(value);
            return value;
        });
    }

    public synchronized CompletableFuture<StringValue> getControllerVersion() {
        var cached = this.controllerVersion.getValue();
        if (cached.isPresent()) {
            return CompletableFuture.completedFuture(cached.get());
        }
        var observer = new StreamObserverImpl<StringValue>();
        this.client.getControllerVersion(Empty.getDefaultInstance(), observer);
        return observer.future().thenApply((value) -> {
            this.controllerVersion.setValue(value);
            return value;
        });
    }

    @Contract(" -> new")
    public static @NotNull CloudConnection createFromEnv() {
        var address = System.getenv("CONTROLLER_ADDRESS");
        var token = System.getenv("UNIT_TOKEN");
        if (address == null) {
            throw new IllegalStateException("CONTROLLER_ADDRESS not set");
        } else if (token == null) {
            throw new IllegalStateException("UNIT_TOKEN not set");
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
