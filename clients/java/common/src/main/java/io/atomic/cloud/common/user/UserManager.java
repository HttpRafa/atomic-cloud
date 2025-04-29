package io.atomic.cloud.common.user;

import com.google.protobuf.UInt32Value;
import io.atomic.cloud.api.user.Users;
import io.atomic.cloud.common.connection.client.ClientConnection;
import java.util.concurrent.CompletableFuture;
import lombok.AllArgsConstructor;

@AllArgsConstructor
public class UserManager implements Users {

    private final ClientConnection connection;

    @Override
    public CompletableFuture<Integer> userCount() {
        return this.connection.userCount().thenApply(UInt32Value::getValue);
    }
}
