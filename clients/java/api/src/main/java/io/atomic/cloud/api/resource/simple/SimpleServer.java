package io.atomic.cloud.api.resource.simple;

import io.atomic.cloud.api.resource.Resource;
import java.util.Optional;
import java.util.UUID;

public interface SimpleServer extends Resource {

    String name();

    UUID uuid();

    Optional<String> group();

    String node();
}
