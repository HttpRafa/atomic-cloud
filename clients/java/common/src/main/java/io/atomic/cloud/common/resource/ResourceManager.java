package io.atomic.cloud.common.resource;

import io.atomic.cloud.api.resource.Resources;
import io.atomic.cloud.api.resource.simple.SimpleGroup;
import io.atomic.cloud.api.resource.simple.SimpleServer;
import io.atomic.cloud.common.connection.client.ClientConnection;
import io.atomic.cloud.common.resource.object.simple.SimpleGroupImpl;
import io.atomic.cloud.common.resource.object.simple.SimpleServerImpl;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import lombok.AllArgsConstructor;

@AllArgsConstructor
public class ResourceManager implements Resources {

    private final ClientConnection connection;

    @Override
    public CompletableFuture<SimpleGroup[]> groups() {
        return this.connection.groups().thenApply(list -> list.getGroupsList().stream()
                .map(SimpleGroupImpl::new)
                .toArray(SimpleGroupImpl[]::new));
    }

    @Override
    public CompletableFuture<SimpleServer[]> servers() {
        return this.connection.servers().thenApply(list -> list.getServersList().stream()
                .map(server -> new SimpleServerImpl(
                        server.getName(),
                        UUID.fromString(server.getId()),
                        Optional.of(server.getGroup()),
                        server.getNode()))
                .toArray(SimpleServerImpl[]::new));
    }
}
