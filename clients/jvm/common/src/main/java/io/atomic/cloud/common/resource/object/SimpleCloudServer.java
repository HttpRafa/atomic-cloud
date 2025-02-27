package io.atomic.cloud.common.resource.object;

import io.atomic.cloud.api.resource.object.CloudServer;
import java.util.UUID;

public record SimpleCloudServer(String name, UUID uuid) implements CloudServer {}
