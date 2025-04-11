package io.atomic.cloud.common.resource.object.complex;

import io.atomic.cloud.api.resource.complex.CloudNode;
import io.atomic.cloud.grpc.manage.Node;

public record CloudNodeImpl(String name, String plugin, Node.Capabilities capabilities, String controllerAddress)
        implements CloudNode {}
