package io.atomic.cloud.common.user;

import com.google.protobuf.UInt32Value;
import io.atomic.cloud.api.user.CloudUser;
import io.atomic.cloud.api.user.Users;
import io.atomic.cloud.common.connection.client.ClientConnection;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import lombok.AllArgsConstructor;
import org.jetbrains.annotations.NotNull;

@AllArgsConstructor
public class UserManager implements Users {

    private final ClientConnection connection;

    @Override
    public CompletableFuture<Integer> userCount() {
        return this.connection.userCount().thenApply(UInt32Value::getValue);
    }

    @Override
    public CompletableFuture<Optional<CloudUser>> userFromName(String name) {
        return this.connection
                .userFromName(name)
                .thenApply(user -> {
                    Optional<String> group = Optional.empty();
                    Optional<UUID> server = Optional.empty();
                    if (user.hasGroup()) {
                        group = Optional.of(user.getGroup());
                    }
                    if (user.hasServer()) {
                        server = Optional.of(UUID.fromString(user.getServer()));
                    }
                    return Optional.of((CloudUser)
                            new CloudUserImpl(user.getName(), UUID.fromString(user.getId()), group, server));
                })
                .exceptionally(throwable -> {
                    if (throwable.getMessage().equals("NOT_FOUND")) {
                        return Optional.empty();
                    } else {
                        throw new RuntimeException(throwable);
                    }
                });
    }

    @Override
    public CompletableFuture<Optional<CloudUser>> userFromUuid(@NotNull UUID uuid) {
        return this.connection
                .user(uuid.toString())
                .thenApply(user -> {
                    Optional<String> group = Optional.empty();
                    Optional<UUID> server = Optional.empty();
                    if (user.hasGroup()) {
                        group = Optional.of(user.getGroup());
                    }
                    if (user.hasServer()) {
                        server = Optional.of(UUID.fromString(user.getServer()));
                    }
                    return Optional.of((CloudUser) new CloudUserImpl(user.getName(), uuid, group, server));
                })
                .exceptionally(throwable -> {
                    if (throwable.getMessage().equals("NOT_FOUND")) {
                        return Optional.empty();
                    } else {
                        throw new RuntimeException(throwable);
                    }
                });
    }
}
