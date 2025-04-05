package io.atomic.cloud.common.resource.object.simple;

import io.atomic.cloud.api.resource.simple.SimpleServer;
import java.util.Optional;
import java.util.UUID;

public record SimpleServerImpl(String name, UUID uuid, Optional<String> group, String node) implements SimpleServer {}
