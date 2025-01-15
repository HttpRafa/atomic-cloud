package io.atomic.cloud.api.objects;

import java.util.concurrent.CompletableFuture;

public interface LocalCloudUnit {

    /**
     * Shut down this unit instance.
     * This will stop the unit and transfer all players to a different unit.
     * How the unit is shutdown depends on the disk retention policy.
     * If the unit is marked as permanent, it will not be deleted.
     * If the unit is not marked as permanent, it will be killed and deleted.
     * @return a future to be completed once the unit has been shut down
     */
    CompletableFuture<Void> shutdown();

    /**
     * Mark this unit as ready
     * @return a future to be completed once the unit has been marked as ready
     */
    CompletableFuture<Void> markReady();

    /**
     * Mark this unit as not ready
     * @return a future to be completed once the unit has been marked as not ready
     */
    CompletableFuture<Void> markNotReady();
}
