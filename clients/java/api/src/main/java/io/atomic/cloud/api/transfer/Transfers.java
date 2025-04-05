package io.atomic.cloud.api.transfer;

import io.atomic.cloud.api.resource.simple.SimpleGroup;
import io.atomic.cloud.api.resource.simple.SimpleServer;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;

public interface Transfers {

    /**
     * Sends a request to the controller to transfer the specified users to a new server.
     *
     * @param server The target server to which the users should be transferred.
     * @param userUUID A list of user UUIDs to transfer. These users must belong to the current
     *     server; otherwise, the controller will return an error.
     * @return The number of users successfully transferred.
     */
    CompletableFuture<Integer> transferUsersToServer(SimpleServer server, UUID... userUUID);

    /**
     * Sends a request to the controller to transfer the specified users to a new server on specific
     * group.
     *
     * @param group The target group to which the users should be transferred.
     * @param userUUID A list of user UUIDs to transfer. These users must belong to the current
     *     server; otherwise, the controller will return an error.
     * @return The number of users successfully transferred.
     */
    CompletableFuture<Integer> transferUsersToGroup(SimpleGroup group, UUID... userUUID);

    /**
     * Sends a request to the controller to transfer the specified users to a new server marked as
     * fallback.
     *
     * @param userUUID A list of user UUIDs to transfer. These users must belong to the current
     *     server; otherwise, the controller will return an error.
     * @return The number of users successfully transferred.
     */
    CompletableFuture<Integer> transferUsersToFallback(UUID... userUUID);
}
