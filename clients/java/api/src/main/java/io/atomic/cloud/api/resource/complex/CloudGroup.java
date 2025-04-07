package io.atomic.cloud.api.resource.complex;

import io.atomic.cloud.api.resource.simple.SimpleCloudGroup;
import io.atomic.cloud.grpc.manage.Group;
import io.atomic.cloud.grpc.manage.Server;

public interface CloudGroup extends SimpleCloudGroup {

    String[] nodes();

    Group.Constraints constraints();

    Group.Scaling scaling();

    Server.Resources resources();

    Server.Spec spec();
}
