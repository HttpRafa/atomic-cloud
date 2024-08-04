package io.atomic.cloud.paper.api;

import io.atomic.cloud.api.Cloud;
import io.atomic.cloud.api.channel.Channels;
import io.atomic.cloud.api.server.CloudServer;
import io.atomic.cloud.paper.CloudPlugin;

public class CloudImpl implements Cloud.CloudAPI {

    @Override
    public CloudServer self() {
        return CloudPlugin.INSTANCE.self();
    }

    @Override
    public Channels channels() {
        return CloudPlugin.INSTANCE.channels();
    }

    @Override
    public void disableAutoReady() {
        CloudPlugin.INSTANCE.settings().autoReady(false);
    }

}
