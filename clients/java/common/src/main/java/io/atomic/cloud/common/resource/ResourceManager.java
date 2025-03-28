package io.atomic.cloud.common.resource;

import io.atomic.cloud.api.resource.Resources;
import io.atomic.cloud.api.resource.object.CloudGroup;
import io.atomic.cloud.api.resource.object.CloudServer;
import io.atomic.cloud.common.connection.CloudConnection;
import io.atomic.cloud.common.resource.object.SimpleCloudGroup;
import io.atomic.cloud.common.resource.object.SimpleCloudServer;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import lombok.AllArgsConstructor;

@AllArgsConstructor
public class ResourceManager implements Resources {

    private final CloudConnection connection;

    @Override
    public CompletableFuture<CloudGroup[]> groups() {
        return this.connection.getGroups().thenApply(list -> list.getGroupsList().stream()
                .map(SimpleCloudGroup::new)
                .toArray(SimpleCloudGroup[]::new));
    }

    @Override
    public CompletableFuture<CloudServer[]> servers() {
        return this.connection.getServers().thenApply(list -> list.getServersList().stream()
                .map(server -> new SimpleCloudServer(server.getName(), UUID.fromString(server.getId())))
                .toArray(SimpleCloudServer[]::new));
    }
}
