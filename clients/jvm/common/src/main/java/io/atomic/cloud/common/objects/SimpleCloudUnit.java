package io.atomic.cloud.common.objects;

import io.atomic.cloud.api.objects.CloudUnit;
import java.util.UUID;
import lombok.AllArgsConstructor;
import lombok.Getter;

@AllArgsConstructor
@Getter
public class SimpleCloudUnit implements CloudUnit {

    protected final String name;
    protected final UUID uuid;
}
