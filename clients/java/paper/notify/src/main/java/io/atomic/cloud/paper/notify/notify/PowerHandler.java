package io.atomic.cloud.paper.notify.notify;

import io.atomic.cloud.common.connection.call.CallHandle;
import io.atomic.cloud.common.connection.client.ClientConnection;
import io.atomic.cloud.grpc.common.Notify;
import io.atomic.cloud.paper.CloudPlugin;
import io.atomic.cloud.paper.notify.NotifyPlugin;
import io.atomic.cloud.paper.notify.permission.Permissions;
import io.grpc.stub.StreamObserver;
import lombok.RequiredArgsConstructor;
import net.kyori.adventure.text.minimessage.tag.resolver.Placeholder;
import org.bukkit.Bukkit;

@RequiredArgsConstructor
public class PowerHandler implements StreamObserver<Notify.PowerEvent> {

    private final ClientConnection connection;
    private CallHandle<?, ?> handle;

    public void enable() {
        NotifyPlugin.LOGGER.info("Enabling power notifications...");
        this.handle = this.connection.subscribeToPowerEvents(this);
    }

    public void cleanup() {
        this.handle.cancel();
    }

    @Override
    public void onNext(Notify.PowerEvent powerEvent) {
        try {
            Bukkit.getOnlinePlayers().stream()
                    .filter(Permissions.POWER_NOTIFY::check)
                    .forEach(player -> {
                        if (powerEvent.getState() == Notify.PowerEvent.State.START) {
                            NotifyPlugin.INSTANCE
                                    .messages()
                                    .serverStarting()
                                    .send(
                                            player,
                                            Placeholder.unparsed("name", powerEvent.getName()),
                                            Placeholder.unparsed("node", powerEvent.getNode()));
                        } else {
                            NotifyPlugin.INSTANCE
                                    .messages()
                                    .serverStopping()
                                    .send(
                                            player,
                                            Placeholder.unparsed("name", powerEvent.getName()),
                                            Placeholder.unparsed("node", powerEvent.getNode()));
                        }
                    });
        } catch (Throwable throwable) {
            NotifyPlugin.LOGGER.info(
                    "Failed to process power event for server {}: {}", powerEvent.getName(), throwable);
        }
    }

    @Override
    public void onError(Throwable throwable) {
        CloudPlugin.LOGGER.error("Failed to handle power event", throwable);
    }

    @Override
    public void onCompleted() {}
}
