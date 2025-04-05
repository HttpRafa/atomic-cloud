package io.atomic.cloud.api.resource.complex;

import io.atomic.cloud.api.resource.simple.SimpleNode;
import java.util.Optional;

public interface Node extends SimpleNode {

    String plugin();

    Optional<Integer> memory();

    Optional<Integer> maxServers();

    Optional<String> child();

    String controllerAddress();
}
