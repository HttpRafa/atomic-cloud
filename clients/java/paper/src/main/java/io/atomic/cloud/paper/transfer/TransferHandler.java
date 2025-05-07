package io.atomic.cloud.paper.transfer;

import io.atomic.cloud.common.connection.call.CallHandle;
import io.atomic.cloud.common.connection.client.ClientConnection;
import io.atomic.cloud.grpc.client.Transfer;
import io.atomic.cloud.paper.CloudPlugin;
import io.grpc.stub.StreamObserver;
import java.util.UUID;
import lombok.RequiredArgsConstructor;
import org.jetbrains.annotations.NotNull;

@RequiredArgsConstructor
public class TransferHandler implements StreamObserver<Transfer.TransferRes> {

    private final ClientConnection connection;
    private CallHandle<?, ?> handle;

    public void enable() {
        CloudPlugin.LOGGER.info("Enabling transfer system...");
        this.handle = this.connection.subscribeToTransfers(this);
    }

    public void cleanup() {
        this.handle.cancel("Closed by cleanup");
    }

    @Override
    public void onNext(Transfer.TransferRes resolvedTransfer) {
        try {
            var uuid = UUID.fromString(resolvedTransfer.getId());
            var player = CloudPlugin.INSTANCE.getServer().getPlayer(uuid);
            if (player == null) {
                CloudPlugin.LOGGER.error(
                        "Failed to handle transfer request for user {}: Player not found", resolvedTransfer.getId());
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
                    "Failed to handle transfer request for user {}: {}", resolvedTransfer.getId(), throwable);
        }
    }

    @Override
    public void onError(@NotNull Throwable throwable) {
        if (throwable.getMessage().contains("CANCELLED")) {
            return;
        }
        CloudPlugin.LOGGER.error("Failed to handle transfer request", throwable);
    }

    @Override
    public void onCompleted() {}
}
