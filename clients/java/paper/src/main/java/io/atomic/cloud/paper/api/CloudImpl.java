package io.atomic.cloud.paper.api;

import io.atomic.cloud.api.Cloud;
import io.atomic.cloud.api.client.channel.Channels;
import io.atomic.cloud.api.client.self.LocalCloudServer;
import io.atomic.cloud.api.manage.Privileged;
import io.atomic.cloud.api.resource.Resources;
import io.atomic.cloud.api.transfer.Transfers;
import io.atomic.cloud.common.connection.client.ManageConnection;
import io.atomic.cloud.common.connection.impl.PrivilegedImpl;
import io.atomic.cloud.paper.CloudPlugin;
import java.io.IOException;

public class CloudImpl implements Cloud.CloudAPI {

    @Override
    public LocalCloudServer self() {
        return CloudPlugin.INSTANCE.self();
    }

    @Override
    public Resources resources() {
        return CloudPlugin.INSTANCE.resources();
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
    public Privileged privileged(String token) throws IOException {
        var connection = ManageConnection.createFromOther(CloudPlugin.INSTANCE.clientConnection(), token);
        connection.connect();
        return new PrivilegedImpl(connection);
    }

    @Override
    public void disableAutoReady() {
        CloudPlugin.INSTANCE.settings().autoReady(false);
    }
}
