package io.atomic.cloud.common.resource.object.complex;

import io.atomic.cloud.api.resource.complex.Node;
import java.util.Optional;

public record NodeImpl(
        String name,
        String plugin,
        Optional<Integer> memory,
        Optional<Integer> maxServers,
        Optional<String> child,
        String controllerAddress)
        implements Node {}
