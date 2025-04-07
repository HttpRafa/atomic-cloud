package io.atomic.cloud.api.resource.complex;

import io.atomic.cloud.api.resource.simple.SimpleCloudNode;
import java.util.Optional;

public interface CloudNode extends SimpleCloudNode {

    String plugin();

    Optional<Integer> memory();

    Optional<Integer> maxServers();

    Optional<String> child();

    String controllerAddress();
}
