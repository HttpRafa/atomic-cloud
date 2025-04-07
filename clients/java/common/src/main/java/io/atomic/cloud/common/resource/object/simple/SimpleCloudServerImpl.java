package io.atomic.cloud.common.resource.object.simple;

import io.atomic.cloud.api.resource.simple.SimpleCloudServer;
import java.util.Optional;
import java.util.UUID;

public record SimpleCloudServerImpl(String name, UUID uuid, Optional<String> group, String node)
        implements SimpleCloudServer {}
