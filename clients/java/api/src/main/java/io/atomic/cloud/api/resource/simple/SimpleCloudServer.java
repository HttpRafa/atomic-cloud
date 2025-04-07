package io.atomic.cloud.api.resource.simple;

import io.atomic.cloud.api.resource.CloudResource;
import java.util.Optional;
import java.util.UUID;

public interface SimpleCloudServer extends CloudResource {

    String name();

    UUID uuid();

    Optional<String> group();

    String node();
}
