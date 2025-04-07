package io.atomic.cloud.common.resource;

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

@AllArgsConstructor
public class ResourceManager implements Resources {

    private final ClientConnection connection;

    @Override
    public CompletableFuture<SimpleCloudGroup[]> groups() {
        return this.connection.groups().thenApply(list -> list.getGroupsList().stream()
                .map(SimpleCloudGroupImpl::new)
                .toArray(SimpleCloudGroupImpl[]::new));
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
}
