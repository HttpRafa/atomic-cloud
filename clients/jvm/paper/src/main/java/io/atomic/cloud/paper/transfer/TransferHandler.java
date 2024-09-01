package io.atomic.cloud.paper.transfer;

import io.atomic.cloud.common.connection.CloudConnection;
import io.atomic.cloud.grpc.server.ResolvedTransfer;
import io.atomic.cloud.paper.CloudPlugin;
import io.grpc.stub.StreamObserver;
import java.util.UUID;
import lombok.RequiredArgsConstructor;

@RequiredArgsConstructor
public class TransferHandler implements StreamObserver<ResolvedTransfer> {

    private final CloudConnection cloudConnection;

    public void enable() {
        CloudPlugin.LOGGER.info("Enabling transfer system...");
        this.cloudConnection.subscribeToTransfers(this);
    }

    @Override
    public void onNext(ResolvedTransfer resolvedTransfer) {
        try {
            var uuid = UUID.fromString(resolvedTransfer.getUser().getUuid());
            var player = CloudPlugin.INSTANCE.getServer().getPlayer(uuid);
            if (player == null) {
                CloudPlugin.LOGGER.error(
                        "Failed to handle transfer request for user {}: Player not found",
                        resolvedTransfer.getUser().getUuid());
                return;
            }

            player.transfer(resolvedTransfer.getHost(), resolvedTransfer.getPort());
            CloudPlugin.LOGGER.info(
                    "Transferred user {} to {}:{}",
                    player.getName(),
                    resolvedTransfer.getHost(),
                    resolvedTransfer.getPort());
        } catch (Throwable throwable) {
            CloudPlugin.LOGGER.error(
                    "Failed to handle transfer request for user {}: {}",
                    resolvedTransfer.getUser().getUuid(),
                    throwable);
        }
    }

    @Override
    public void onError(Throwable throwable) {
        CloudPlugin.LOGGER.error("Failed to handle transfer request", throwable);
    }

    @Override
    public void onCompleted() {}
}
