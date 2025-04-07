package io.atomic.cloud.common.resource.object.complex;

import io.atomic.cloud.api.resource.complex.CloudServer;
import io.atomic.cloud.grpc.manage.Server;
import java.util.Optional;
import java.util.UUID;

public record CloudServerImpl(
        String name,
        UUID uuid,
        Optional<String> group,
        String node,
        Server.Allocation allocation,
        int users,
        String token,
        Server.State state,
        boolean ready)
        implements CloudServer {}
