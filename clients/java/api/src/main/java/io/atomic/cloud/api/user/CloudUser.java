package io.atomic.cloud.api.user;

import io.atomic.cloud.api.resource.simple.SimpleCloudGroup;
import io.atomic.cloud.api.resource.simple.SimpleCloudServer;
import java.util.Optional;
import java.util.UUID;

public interface CloudUser {

    String name();

    UUID uuid();

    Optional<SimpleCloudGroup> group();

    Optional<SimpleCloudServer> server();
}
