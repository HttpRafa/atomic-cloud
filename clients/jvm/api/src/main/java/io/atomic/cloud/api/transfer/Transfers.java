package io.atomic.cloud.api.transfer;

import io.atomic.cloud.api.objects.CloudDeployment;
import io.atomic.cloud.api.objects.CloudUnit;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;

public interface Transfers {

    /**
     * Sends a request to the controller to transfer the specified users to a new unit.
     * @param unit The target unit to which the users should be transferred.
     * @param userUUID A list of user UUIDs to transfer. These users must belong to the current unit; otherwise, the controller will return an error.
     * @return The number of users successfully transferred.
     */
    CompletableFuture<Integer> transferUsersToUnit(CloudUnit unit, UUID... userUUID);

    /**
     * Sends a request to the controller to transfer the specified users to a new unit on specific deployment.
     * @param deployment The target deployment to which the users should be transferred.
     * @param userUUID A list of user UUIDs to transfer. These users must belong to the current unit; otherwise, the controller will return an error.
     * @return The number of users successfully transferred.
     */
    CompletableFuture<Integer> transferUsersToDeployment(CloudDeployment deployment, UUID... userUUID);

    /**
     * Sends a request to the controller to transfer the specified users to a new unit marked as fallback.
     * @param userUUID A list of user UUIDs to transfer. These users must belong to the current unit; otherwise, the controller will return an error.
     * @return The number of users successfully transferred.
     */
    CompletableFuture<Integer> transferUsersToFallback(UUID... userUUID);
}
