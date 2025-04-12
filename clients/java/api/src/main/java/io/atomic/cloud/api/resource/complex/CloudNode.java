package io.atomic.cloud.api.resource.complex;

import io.atomic.cloud.api.resource.simple.SimpleCloudNode;
import io.atomic.cloud.grpc.manage.Node;

public interface CloudNode extends SimpleCloudNode {

    String plugin();

    Node.Capabilities capabilities();

    String controllerAddress();
}
