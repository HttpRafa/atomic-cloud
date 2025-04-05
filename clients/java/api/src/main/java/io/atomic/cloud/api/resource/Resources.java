package io.atomic.cloud.api.resource;

import io.atomic.cloud.api.resource.simple.SimpleGroup;
import io.atomic.cloud.api.resource.simple.SimpleServer;
import java.util.concurrent.CompletableFuture;

/** The Resources interface provides methods to access cloud groups and cloud servers. */
public interface Resources {

    /**
     * Retrieves an array of CloudGroup objects.
     *
     * @return an array of CloudGroup instances
     */
    CompletableFuture<SimpleGroup[]> groups();

    /**
     * Retrieves an array of CloudServer objects.
     *
     * @return an array of CloudServer instances
     */
    CompletableFuture<SimpleServer[]> servers();
}
