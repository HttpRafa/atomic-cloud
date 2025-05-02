package io.atomic.cloud.common.resource;

import io.atomic.cloud.api.Cloud;
import io.atomic.cloud.api.resource.Resources;
import io.atomic.cloud.api.resource.simple.SimpleCloudGroup;
import io.atomic.cloud.api.resource.simple.SimpleCloudServer;
import io.atomic.cloud.common.connection.client.ClientConnection;
import io.atomic.cloud.common.resource.object.simple.SimpleCloudGroupImpl;
import io.atomic.cloud.common.resource.object.simple.SimpleCloudServerImpl;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import lombok.AllArgsConstructor;
import org.jetbrains.annotations.NotNull;

@AllArgsConstructor
public class ResourceManager implements Resources {

    private final ClientConnection connection;

    @Override
    public CompletableFuture<SimpleCloudGroup[]> groups() {
        return this.connection.groups().thenApply(list -> list.getGroupsList().stream()
                .map(group -> new SimpleCloudGroupImpl(group.getName()))
                .toArray(SimpleCloudGroupImpl[]::new));
    }

    @Override
    public CompletableFuture<Optional<SimpleCloudGroup>> groupFromName(String name) {
        return this.connection
                .group(name)
                .thenApply(group -> Optional.of((SimpleCloudGroup) new SimpleCloudGroupImpl(group.getName())))
                .exceptionally(throwable -> {
                    if (throwable.getMessage().equals("NOT_FOUND")) {
                        return Optional.empty();
                    } else {
                        throw new RuntimeException(throwable);
                    }
                });
    }

    @Override
    public CompletableFuture<SimpleCloudServer[]> servers() {
        return this.connection.servers().thenApply(list -> list.getServersList().stream()
                .map(server -> new SimpleCloudServerImpl(
                        server.getName(),
                        UUID.fromString(server.getId()),
                        Optional.of(server.getGroup()),
                        server.getNode()))
                .toArray(SimpleCloudServerImpl[]::new));
    }

    @Override
    public CompletableFuture<Optional<SimpleCloudServer>> serverFromName(String name) {
        return this.connection
                .serverFromName(name)
                .thenApply(server -> {
                    Optional<String> group = Optional.empty();
                    if (server.hasGroup()) {
                        group = Optional.of(server.getGroup());
                    }
                    return Optional.of((SimpleCloudServer) new SimpleCloudServerImpl(
                            server.getName(), UUID.fromString(server.getId()), group, server.getNode()));
                })
                .exceptionally(throwable -> {
                    if (throwable.getMessage().equals("NOT_FOUND")) {
                        return Optional.empty();
                    } else {
                        throw new RuntimeException(throwable);
                    }
                });
    }

    @Override
    public CompletableFuture<Optional<SimpleCloudServer>> serverFromUuid(@NotNull UUID uuid) {
        return this.connection
                .server(uuid.toString())
                .thenApply(server -> {
                    Optional<String> group = Optional.empty();
                    if (server.hasGroup()) {
                        group = Optional.of(server.getGroup());
                    }
                    return Optional.of((SimpleCloudServer)
                            new SimpleCloudServerImpl(server.getName(), uuid, group, server.getNode()));
                })
                .exceptionally(throwable -> {
                    if (throwable.getMessage().equals("NOT_FOUND")) {
                        return Optional.empty();
                    } else {
                        throw new RuntimeException(throwable);
                    }
                });
    }

    @Override
    public CompletableFuture<Optional<SimpleCloudServer>> serverFromUser(UUID uuid) {
        return Cloud.users().userFromUuid(uuid).thenCompose(user -> {
            if (user.isPresent() && user.get().server().isPresent()) {
                return this.serverFromUuid(user.get().server().get());
            } else {
                return CompletableFuture.completedFuture(Optional.empty());
            }
        });
    }
}
