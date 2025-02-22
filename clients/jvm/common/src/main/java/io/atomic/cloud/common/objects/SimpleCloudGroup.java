package io.atomic.cloud.common.objects;

import io.atomic.cloud.api.objects.CloudGroup;
import lombok.AllArgsConstructor;
import lombok.Getter;

@AllArgsConstructor
@Getter
public class SimpleCloudGroup implements CloudGroup {

    protected final String name;
}
