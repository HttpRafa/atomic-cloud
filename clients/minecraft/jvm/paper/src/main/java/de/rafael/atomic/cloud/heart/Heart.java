package de.rafael.atomic.cloud.heart;

import de.rafael.atomic.cloud.CloudPlugin;
import io.papermc.paper.threadedregions.scheduler.ScheduledTask;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.Consumer;
import lombok.RequiredArgsConstructor;
import org.bukkit.Bukkit;

@RequiredArgsConstructor
public class Heart implements Consumer<ScheduledTask> {

  private final long interval;
  private final AtomicBoolean running = new AtomicBoolean(false);

  public void start() {
    CloudPlugin.LOGGER.info("Starting heart of this server...");
    running.set(true);
    Bukkit.getAsyncScheduler()
        .runAtFixedRate(CloudPlugin.INSTANCE, this, 0, interval, TimeUnit.SECONDS);
  }

  public void stop() {
    CloudPlugin.LOGGER.info("Stopping heart of this server...");
    running.set(false);
  }

  @Override
  public void accept(ScheduledTask scheduledTask) {
    if (!running.get()) {
      scheduledTask.cancel();
      return;
    }
    CloudPlugin.INSTANCE.getConnection().beatHeart();
  }
}
