package de.rafael.atomic.cloud

import io.papermc.paper.plugin.bootstrap.BootstrapContext
import io.papermc.paper.plugin.bootstrap.PluginBootstrap
import io.papermc.paper.plugin.bootstrap.PluginProviderContext
import org.bukkit.plugin.java.JavaPlugin

class PluginBootstrap : PluginBootstrap {

    override fun bootstrap(bootstrap: BootstrapContext) {}

    override fun createPlugin(context: PluginProviderContext): JavaPlugin {
        return Plugin
    }
}
