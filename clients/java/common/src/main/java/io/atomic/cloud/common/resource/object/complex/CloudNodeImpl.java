package io.atomic.cloud.common.resource.object.complex;

import io.atomic.cloud.api.resource.complex.CloudNode;
import java.util.Optional;

public record CloudNodeImpl(
        String name,
        String plugin,
        Optional<Integer> memory,
        Optional<Integer> maxServers,
        Optional<String> child,
        String controllerAddress)
        implements CloudNode {}
