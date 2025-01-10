package io.atomic.cloud.common.objects;

import io.atomic.cloud.api.objects.CloudDeployment;
import lombok.AllArgsConstructor;
import lombok.Getter;

@AllArgsConstructor
@Getter
public class SimpleCloudDeployment implements CloudDeployment {

    protected final String name;
}
