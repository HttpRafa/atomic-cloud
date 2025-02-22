package io.atomic.cloud.common.objects;

import io.atomic.cloud.api.objects.CloudServer;
import java.util.UUID;
import lombok.AllArgsConstructor;
import lombok.Getter;

@AllArgsConstructor
@Getter
public class SimpleCloudServer implements CloudServer {

    protected final String name;
    protected final UUID uuid;
}
