package io.atomic.cloud.common.resource.object.complex;

import io.atomic.cloud.api.resource.complex.CloudGroup;
import io.atomic.cloud.grpc.manage.Group;
import io.atomic.cloud.grpc.manage.Server;

public record CloudGroupImpl(
        String name,
        String[] nodes,
        Group.Constraints constraints,
        Group.Scaling scaling,
        Server.Resources resources,
        Server.Specification specification)
        implements CloudGroup {}
