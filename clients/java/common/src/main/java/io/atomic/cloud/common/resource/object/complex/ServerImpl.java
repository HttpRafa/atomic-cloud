package io.atomic.cloud.common.resource.object.complex;

import io.atomic.cloud.api.resource.complex.Server;
import java.util.Optional;
import java.util.UUID;

public record ServerImpl(
        String name, UUID uuid, Optional<String> group, String node, int users, String token, boolean ready)
        implements Server {}
