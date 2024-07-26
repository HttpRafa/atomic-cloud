package de.rafael.atomic.cloud.api;

import de.rafael.atomic.cloud.CloudPlugin;
import java.util.concurrent.CompletableFuture;

public class CloudImpl implements Cloud.CloudInterface {

  @Override
  public void disableAutoReady() {
    CloudPlugin.INSTANCE.getSettings().setAutoReady(false);
  }

  @Override
  public CompletableFuture<Void> markReady() {
    return CloudPlugin.INSTANCE.getConnection().markReady().thenApply(empty -> null);
  }

  @Override
  public CompletableFuture<Void> markNotReady() {
    return CloudPlugin.INSTANCE.getConnection().markReady().thenApply(empty -> null);
  }
}
