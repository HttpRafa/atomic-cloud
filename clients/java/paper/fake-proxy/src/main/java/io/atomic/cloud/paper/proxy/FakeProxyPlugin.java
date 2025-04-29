package io.atomic.cloud.paper.proxy;

import io.atomic.cloud.paper.proxy.listener.ServerListPingListener;
import io.atomic.cloud.paper.proxy.setting.Settings;
import lombok.Getter;
import org.bukkit.plugin.java.JavaPlugin;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Getter
public class FakeProxyPlugin extends JavaPlugin {

    public static final FakeProxyPlugin INSTANCE = new FakeProxyPlugin();
    public static final Logger LOGGER = LoggerFactory.getLogger("ac-fake-proxy");

    private Settings settings;

    @Override
    public void onLoad() {
        // Load configuration
        saveDefaultConfig();
        this.settings = new Settings(this.getConfig());
    }

    @Override
    public void onEnable() {
        // Register listeners
        registerListeners();
    }

    private void registerListeners() {
        var pluginManager = getServer().getPluginManager();
        if (this.settings.changeOnlinePlayers()) {
            pluginManager.registerEvents(new ServerListPingListener(), this);
        }
    }
}
