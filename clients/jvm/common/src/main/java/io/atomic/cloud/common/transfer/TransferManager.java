package io.atomic.cloud.common.transfer;

import com.google.protobuf.UInt32Value;
import io.atomic.cloud.api.objects.CloudDeployment;
import io.atomic.cloud.api.objects.CloudUnit;
import io.atomic.cloud.api.transfer.Transfers;
import io.atomic.cloud.common.connection.CloudConnection;
import io.atomic.cloud.grpc.server.TransferManagement;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import lombok.AllArgsConstructor;
import org.jetbrains.annotations.NotNull;

@AllArgsConstructor
public class TransferManager implements Transfers {

    private final CloudConnection connection;

    @Override
    public CompletableFuture<Integer> transferUsersToUnit(@NotNull CloudUnit server, UUID @NotNull ... userUUID) {
        var builder = TransferManagement.TransferUsersRequest.newBuilder();
        builder.setTarget(TransferManagement.TransferTargetValue.newBuilder()
                .setTargetType(TransferManagement.TransferTargetValue.TargetType.UNIT)
                .setTarget(server.uuid().toString())
                .build());
        for (UUID uuid : userUUID) {
            builder.addUserUuids(uuid.toString());
        }
        return this.connection.transferUsers(builder.build()).thenApply(UInt32Value::getValue);
    }

    @Override
    public CompletableFuture<Integer> transferUsersToDeployment(
            @NotNull CloudDeployment group, UUID @NotNull ... userUUID) {
        var builder = TransferManagement.TransferUsersRequest.newBuilder();
        builder.setTarget(TransferManagement.TransferTargetValue.newBuilder()
                .setTargetType(TransferManagement.TransferTargetValue.TargetType.DEPLOYMENT)
                .setTarget(group.name())
                .build());
        for (UUID uuid : userUUID) {
            builder.addUserUuids(uuid.toString());
        }
        return this.connection.transferUsers(builder.build()).thenApply(UInt32Value::getValue);
    }

    @Override
    public CompletableFuture<Integer> transferUsersToFallback(UUID @NotNull ... userUUID) {
        var builder = TransferManagement.TransferUsersRequest.newBuilder();
        builder.setTarget(TransferManagement.TransferTargetValue.newBuilder()
                .setTargetType(TransferManagement.TransferTargetValue.TargetType.FALLBACK)
                .build());
        for (UUID uuid : userUUID) {
            builder.addUserUuids(uuid.toString());
        }
        return this.connection.transferUsers(builder.build()).thenApply(UInt32Value::getValue);
    }
}
