package io.atomic.cloud.api.resource;

import io.atomic.cloud.api.resource.simple.SimpleCloudGroup;
import io.atomic.cloud.api.resource.simple.SimpleCloudServer;
import java.util.Collection;
import java.util.Optional;
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
     * Retrieves a SimpleGroup object that contains the specified user as a member.
     *
     * @param uuid the uuid of the user to retrieve the group for
     * @return a SimpleGroup instance
     */
    CompletableFuture<Optional<SimpleCloudGroup>> groupFromUser(String uuid);

    /**
     * Retrieves an array of SimpleServer objects.
     *
     * @return an array of SimpleServer instances
     */
    CompletableFuture<SimpleCloudServer[]> servers();

    /**
     * Retrieves an array of SimpleServer objects that match the specified name.
     * <p>
     * NOTE: It is possible that multiple servers have the same name.
     * Because the controller currently on imposes no uniqueness constraints, this method may return multiple servers.
     *
     * @param name the name of the servers to retrieve
     * @return a collection of SimpleServer instances
     */
    CompletableFuture<Collection<SimpleCloudServer>> serverFromName(String name);

    /**
     * Retrieves a SimpleServer object that matches the specified uuid.
     *
     * @param uuid the uuid of the server to retrieve
     * @return a SimpleServer instance
     */
    CompletableFuture<Optional<SimpleCloudServer>> serverFromUuid(String uuid);

    /**
     * Retrieves a SimpleServer object that contains the specified user as a member.
     *
     * @param uuid the uuid of the user to retrieve the server for
     * @return a SimpleServer instance
     */
    CompletableFuture<Optional<SimpleCloudServer>> serverFromUser(String uuid);
}
