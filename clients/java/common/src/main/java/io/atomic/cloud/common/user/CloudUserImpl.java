package io.atomic.cloud.common.user;

import io.atomic.cloud.api.user.CloudUser;
import java.util.Optional;
import java.util.UUID;

public record CloudUserImpl(String name, UUID uuid, Optional<String> group, Optional<UUID> server)
        implements CloudUser {}
