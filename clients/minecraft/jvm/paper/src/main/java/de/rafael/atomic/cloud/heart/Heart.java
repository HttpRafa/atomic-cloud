package de.rafael.atomic.cloud.heart;

import de.rafael.atomic.cloud.CloudPlugin;
import java.util.concurrent.ScheduledFuture;
import java.util.concurrent.TimeUnit;
import lombok.RequiredArgsConstructor;

@RequiredArgsConstructor
public class Heart {

    private final long interval;
    private ScheduledFuture<?> future;

    public void start() {
        CloudPlugin.LOGGER.info("Starting heart of this server...");
        this.future = CloudPlugin.SCHEDULER.scheduleAtFixedRate(this::beat, 0, interval, TimeUnit.SECONDS);
    }

    public void stop() {
        CloudPlugin.LOGGER.info("Stopping heart of this server...");
        this.future.cancel(false);
    }

    public void beat() {
        CloudPlugin.INSTANCE.getConnection().beatHeart();
    }
}
