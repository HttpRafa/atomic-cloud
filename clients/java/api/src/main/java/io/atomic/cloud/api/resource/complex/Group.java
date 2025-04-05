package io.atomic.cloud.api.resource.complex;

import io.atomic.cloud.api.resource.simple.SimpleGroup;

public interface Group extends SimpleGroup {

    String[] nodes();
}
