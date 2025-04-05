package io.atomic.cloud.common.connection.impl;

import com.google.protobuf.ByteString;
import io.atomic.cloud.api.manage.Privileged;
import io.atomic.cloud.api.resource.Resource;
import io.atomic.cloud.api.resource.complex.Group;
import io.atomic.cloud.api.resource.complex.Node;
import io.atomic.cloud.api.resource.complex.Server;
import io.atomic.cloud.api.resource.simple.SimpleGroup;
import io.atomic.cloud.api.resource.simple.SimpleNode;
import io.atomic.cloud.api.resource.simple.SimpleServer;
import io.atomic.cloud.api.transfer.Transfers;
import io.atomic.cloud.common.connection.client.ManageConnection;
import io.atomic.cloud.common.resource.object.complex.GroupImpl;
import io.atomic.cloud.common.resource.object.complex.NodeImpl;
import io.atomic.cloud.common.resource.object.complex.ServerImpl;
import io.atomic.cloud.common.transfer.ManageTransfers;
import io.atomic.cloud.grpc.manage.Screen;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import lombok.Getter;
import lombok.RequiredArgsConstructor;
import org.jetbrains.annotations.NotNull;

@RequiredArgsConstructor
@Getter
public class PrivilegedImpl implements Privileged {

    private final ManageConnection connection;

    @Override
    public Transfers transfers() {
        return new ManageTransfers(this.connection);
    }

    @Override
    public CompletableFuture<Void> stopController() {
        return this.connection.requestStop().thenAccept(response -> {});
    }

    @Override
    public CompletableFuture<Void> setResource(Resource resource, boolean active) {
        var builder = io.atomic.cloud.grpc.manage.Resource.SetReq.newBuilder();
        builder.setActive(active);
        if (resource instanceof SimpleNode node) {
            builder.setCategory(io.atomic.cloud.grpc.manage.Resource.Category.NODE);
            builder.setId(node.name());
        } else if (resource instanceof SimpleGroup group) {
            builder.setCategory(io.atomic.cloud.grpc.manage.Resource.Category.GROUP);
            builder.setId(group.name());
        } else {
            return CompletableFuture.failedFuture(new UnsupportedOperationException());
        }
        return this.connection.setResource(builder.build()).thenAccept(response -> {});
    }

    @Override
    public CompletableFuture<Void> deleteResource(Resource resource) {
        var builder = io.atomic.cloud.grpc.manage.Resource.DelReq.newBuilder();
        if (resource instanceof SimpleNode node) {
            builder.setCategory(io.atomic.cloud.grpc.manage.Resource.Category.NODE);
            builder.setId(node.name());
        } else if (resource instanceof SimpleGroup group) {
            builder.setCategory(io.atomic.cloud.grpc.manage.Resource.Category.GROUP);
            builder.setId(group.name());
        } else if (resource instanceof SimpleServer server) {
            builder.setCategory(io.atomic.cloud.grpc.manage.Resource.Category.SERVER);
            builder.setId(server.uuid().toString());
        }
        return this.connection.deleteResource(builder.build()).thenAccept(response -> {});
    }

    @Override
    public CompletableFuture<Void> createNode(Node node) {
        return CompletableFuture.failedFuture(new UnsupportedOperationException());
    }

    @Override
    public CompletableFuture<Void> createGroup(Group group) {
        return CompletableFuture.failedFuture(new UnsupportedOperationException());
    }

    @Override
    public CompletableFuture<Void> writeToScreen(@NotNull SimpleServer server, byte[] data) {
        var builder = Screen.WriteReq.newBuilder();
        builder.setId(server.uuid().toString());
        builder.setData(ByteString.copyFrom(data));
        return this.connection.writeToScreen(builder.build()).thenAccept(response -> {});
    }

    @Override
    public CompletableFuture<Node> node(@NotNull SimpleNode node) {
        return this.connection.node(node.name()).thenApply(item -> {
            Optional<Integer> memory = Optional.empty();
            Optional<Integer> maxServers = Optional.empty();
            Optional<String> child = Optional.empty();
            if (item.hasMemory()) {
                memory = Optional.of(item.getMemory());
            }
            if (item.hasMax()) {
                maxServers = Optional.of(item.getMax());
            }
            if (item.hasMax()) {
                child = Optional.of(item.getChild());
            }
            return new NodeImpl(item.getName(), item.getPlugin(), memory, maxServers, child, item.getCtrlAddr());
        });
    }

    @Override
    public CompletableFuture<Group> group(@NotNull SimpleGroup group) {
        return this.connection
                .group(group.name())
                .thenApply(item ->
                        new GroupImpl(item.getName(), item.getNodesList().toArray(new String[0])));
    }

    @Override
    public CompletableFuture<Server> server(@NotNull SimpleServer server) {
        return this.connection.server(server.name()).thenApply(item -> {
            Optional<String> group = Optional.empty();
            if (item.hasGroup()) {
                group = Optional.of(item.getGroup());
            }
            return new ServerImpl(
                    item.getName(),
                    UUID.fromString(item.getId()),
                    group,
                    item.getNode(),
                    item.getUsers(),
                    item.getToken(),
                    item.getReady());
        });
    }
}
