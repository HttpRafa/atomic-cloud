package io.atomic.cloud.paper.notify.handler;

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
public class ReadyHandler implements StreamObserver<Notify.ReadyEvent> {

    private final ClientConnection connection;
    private CallHandle<?, ?> handle;

    public void enable() {
        NotifyPlugin.LOGGER.info("Enabling ready notifications...");
        this.handle = this.connection.subscribeToReadyEvents(this);
    }

    public void cleanup() {
        this.handle.cancel("Closed by cleanup");
    }

    @Override
    public void onNext(Notify.ReadyEvent event) {
        try {
            Bukkit.getOnlinePlayers().stream()
                    .filter(Permissions.READY_NOTIFY::check)
                    .forEach(player -> {
                        if (event.getReady()) {
                            NotifyPlugin.INSTANCE
                                    .messages()
                                    .serverReady()
                                    .send(player, Placeholder.unparsed("name", event.getName()));
                        } else {
                            NotifyPlugin.INSTANCE
                                    .messages()
                                    .serverNotReady()
                                    .send(player, Placeholder.unparsed("name", event.getName()));
                        }
                    });
        } catch (Throwable throwable) {
            NotifyPlugin.LOGGER.info("Failed to process ready event for server {}:", event.getName());
            NotifyPlugin.LOGGER.error("-> ", throwable);
        }
    }

    @Override
    public void onError(Throwable throwable) {
        CloudPlugin.LOGGER.error("Failed to handle ready event", throwable);
    }

    @Override
    public void onCompleted() {}
}
