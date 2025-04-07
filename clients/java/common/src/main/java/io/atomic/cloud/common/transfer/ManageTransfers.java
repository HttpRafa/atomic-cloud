package io.atomic.cloud.common.transfer;

import com.google.protobuf.UInt32Value;
import io.atomic.cloud.api.resource.simple.SimpleCloudGroup;
import io.atomic.cloud.api.resource.simple.SimpleCloudServer;
import io.atomic.cloud.api.transfer.Transfers;
import io.atomic.cloud.common.connection.client.ManageConnection;
import io.atomic.cloud.grpc.manage.Transfer;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import lombok.AllArgsConstructor;
import org.jetbrains.annotations.NotNull;

@AllArgsConstructor
public class ManageTransfers implements Transfers {

    private final ManageConnection connection;

    @Override
    public CompletableFuture<Integer> transferUsersToServer(
            @NotNull SimpleCloudServer server, UUID @NotNull ... userUUID) {
        var builder = Transfer.TransferReq.newBuilder();
        builder.setTarget(Transfer.Target.newBuilder()
                .setType(Transfer.Target.Type.SERVER)
                .setTarget(server.uuid().toString())
                .build());
        for (UUID uuid : userUUID) {
            builder.addIds(uuid.toString());
        }
        return this.connection.transferUsers(builder.build()).thenApply(UInt32Value::getValue);
    }

    @Override
    public CompletableFuture<Integer> transferUsersToGroup(
            @NotNull SimpleCloudGroup group, UUID @NotNull ... userUUID) {
        var builder = Transfer.TransferReq.newBuilder();
        builder.setTarget(Transfer.Target.newBuilder()
                .setType(Transfer.Target.Type.GROUP)
                .setTarget(group.name())
                .build());
        for (UUID uuid : userUUID) {
            builder.addIds(uuid.toString());
        }
        return this.connection.transferUsers(builder.build()).thenApply(UInt32Value::getValue);
    }

    @Override
    public CompletableFuture<Integer> transferUsersToFallback(UUID @NotNull ... userUUID) {
        var builder = Transfer.TransferReq.newBuilder();
        builder.setTarget(Transfer.Target.newBuilder()
                .setType(Transfer.Target.Type.FALLBACK)
                .build());
        for (UUID uuid : userUUID) {
            builder.addIds(uuid.toString());
        }
        return this.connection.transferUsers(builder.build()).thenApply(UInt32Value::getValue);
    }
}
