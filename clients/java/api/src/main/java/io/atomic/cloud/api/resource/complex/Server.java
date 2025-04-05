package io.atomic.cloud.api.resource.complex;

import io.atomic.cloud.api.resource.simple.SimpleServer;

public interface Server extends SimpleServer {

    int users();

    String token();

    boolean ready();
}
