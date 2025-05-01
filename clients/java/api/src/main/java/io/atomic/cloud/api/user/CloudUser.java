package io.atomic.cloud.api.user;

import java.util.Optional;
import java.util.UUID;

public interface CloudUser {

    String name();

    UUID uuid();

    Optional<String> group();

    Optional<UUID> server();
}
