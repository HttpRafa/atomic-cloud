package io.atomic.cloud.api.resource;

import io.atomic.cloud.api.resource.simple.SimpleCloudGroup;
import io.atomic.cloud.api.resource.simple.SimpleCloudServer;
import java.util.concurrent.CompletableFuture;

/** The Resources interface provides methods to access cloud groups and cloud servers. */
public interface Resources {

    /**
     * Retrieves an array of SimpleGroup objects.
     *
     * @return an array of SimpleGroup instances
     */
    CompletableFuture<SimpleCloudGroup[]> groups();

    /**
     * Retrieves an array of SimpleServer objects.
     *
     * @return an array of SimpleServer instances
     */
    CompletableFuture<SimpleCloudServer[]> servers();
}
