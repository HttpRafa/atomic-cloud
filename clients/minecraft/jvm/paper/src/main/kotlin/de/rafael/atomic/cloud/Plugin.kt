package de.rafael.atomic.cloud

import de.rafael.atomic.cloud.heart.Heart
import org.bukkit.plugin.java.JavaPlugin
import org.slf4j.Logger
import org.slf4j.LoggerFactory

object Plugin : JavaPlugin() {

    @JvmStatic
    val logger: Logger = LoggerFactory.getLogger("atomic-cloud")

    override fun onLoad() {
        logger.info("Loading cloud client...")
        Heart.start()
    }

    override fun onEnable() {
    }

    override fun onDisable() {
        logger.info("Stopping cloud client...")
        Heart.stop()
    }
}
