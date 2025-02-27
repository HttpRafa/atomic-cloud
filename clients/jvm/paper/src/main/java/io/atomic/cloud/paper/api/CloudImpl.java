package io.atomic.cloud.paper.api;

import io.atomic.cloud.api.Cloud;
import io.atomic.cloud.api.channel.Channels;
import io.atomic.cloud.api.objects.LocalCloudServer;
import io.atomic.cloud.api.transfer.Transfers;
import io.atomic.cloud.paper.CloudPlugin;

public class CloudImpl implements Cloud.CloudAPI {

    @Override
    public LocalCloudServer self() {
        return CloudPlugin.INSTANCE.self();
    }

    @Override
    public Channels channels() {
        return CloudPlugin.INSTANCE.channels();
    }

    @Override
    public Transfers transfers() {
        return CloudPlugin.INSTANCE.transfers();
    }

    @Override
    public void disableAutoReady() {
        CloudPlugin.INSTANCE.settings().autoReady(false);
    }
}
