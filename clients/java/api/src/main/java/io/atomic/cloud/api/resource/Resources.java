package io.atomic.cloud.api.resource;

import io.atomic.cloud.api.resource.simple.SimpleCloudGroup;
import io.atomic.cloud.api.resource.simple.SimpleCloudServer;
import java.util.Optional;
import java.util.UUID;
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
     * Retrieves a SimpleGroup object that matches the specified name.
     *
     * @param name the name of the group to retrieve
     * @return a SimpleGroup instance
     */
    CompletableFuture<Optional<SimpleCloudGroup>> groupFromName(String name);

    /**
     * Retrieves an array of SimpleServer objects.
     *
     * @return an array of SimpleServer instances
     */
    CompletableFuture<SimpleCloudServer[]> servers();

    /**
     * Retrieves a SimpleServer object that matches the specified name.
     *
     * @param name the name of the server to retrieve
     * @return a SimpleServer instance
     */
    CompletableFuture<Optional<SimpleCloudServer>> serverFromName(String name);

    /**
     * Retrieves a SimpleServer object that matches the specified uuid.
     *
     * @param uuid the uuid of the server to retrieve
     * @return a SimpleServer instance
     */
    CompletableFuture<Optional<SimpleCloudServer>> serverFromUuid(UUID uuid);

    /**
     * Retrieves a SimpleServer object that contains the specified user as a member.
     *
     * @param uuid the uuid of the user to retrieve the server for
     * @return a SimpleServer instance
     */
    CompletableFuture<Optional<SimpleCloudServer>> serverFromUser(UUID uuid);
}
