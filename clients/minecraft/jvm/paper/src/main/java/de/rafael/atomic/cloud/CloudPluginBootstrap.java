package de.rafael.atomic.cloud;

import io.papermc.paper.plugin.bootstrap.BootstrapContext;
import io.papermc.paper.plugin.bootstrap.PluginBootstrap;
import io.papermc.paper.plugin.bootstrap.PluginProviderContext;
import org.bukkit.plugin.java.JavaPlugin;
import org.jetbrains.annotations.ApiStatus;
import org.jetbrains.annotations.NotNull;

@ApiStatus.Experimental
public class CloudPluginBootstrap implements PluginBootstrap {

  @Override
  public void bootstrap(@NotNull BootstrapContext bootstrapContext) {}

  @Override
  public @NotNull JavaPlugin createPlugin(@NotNull PluginProviderContext context) {
    return CloudPlugin.INSTANCE;
  }
}
