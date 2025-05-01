package io.atomic.cloud.api.user;

import java.util.UUID;
import java.util.concurrent.CompletableFuture;

/** The Users interface provides methods to access cloud users. */
public interface Users {

    /**
     * Sends a request to the controller to get the number of users currently online
     *
     * @return The number of users on the network
     */
    CompletableFuture<Integer> userCount();

    /**
     * Retrieves a User object that matches the specified name.
     *
     * @param name the uuid of the server to retrieve
     * @return a User instance
     */
    CompletableFuture<CloudUser> userFromName(String name);

    /**
     * Retrieves a User object that matches the specified uuid.
     *
     * @param uuid the uuid of the user to retrieve
     * @return a User instance
     */
    CompletableFuture<CloudUser> userFromUuid(UUID uuid);
}
