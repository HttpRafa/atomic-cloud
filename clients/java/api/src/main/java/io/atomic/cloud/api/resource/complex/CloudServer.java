package io.atomic.cloud.api.resource.complex;

import io.atomic.cloud.api.resource.simple.SimpleCloudServer;
import io.atomic.cloud.grpc.manage.Server;

public interface CloudServer extends SimpleCloudServer {

    Server.Allocation allocation();

    int users();

    String token();

    Server.State state();

    boolean ready();
}
