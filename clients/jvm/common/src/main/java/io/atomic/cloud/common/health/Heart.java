package io.atomic.cloud.common.health;

import io.atomic.cloud.common.connection.CloudConnection;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.ScheduledFuture;
import java.util.concurrent.TimeUnit;
import lombok.RequiredArgsConstructor;

@RequiredArgsConstructor
public class Heart {

    private final long interval;
    private final CloudConnection connection;
    private final ScheduledExecutorService scheduler;
    private ScheduledFuture<?> future;

    public void start() {
        this.future = this.scheduler.scheduleAtFixedRate(this::beat, 0, interval, TimeUnit.SECONDS);
    }

    public void stop() {
        this.future.cancel(false);
    }

    public void beat() {
        this.connection.beat();
    }
}
