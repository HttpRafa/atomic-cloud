package de.rafael.atomic.cloud.heart

import de.rafael.atomic.cloud.Plugin
import io.papermc.paper.threadedregions.scheduler.ScheduledTask
import org.bukkit.Bukkit
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean

object Heart {

    private const val HEARTBEAT_INTERVAL_SEC = 10L // Every 10 seconds
    private val shouldBeat = AtomicBoolean(true)

    fun start() {
        Plugin.logger.info("Starting heart of this server...")
        shouldBeat.set(true)
        Bukkit.getAsyncScheduler().runAtFixedRate(Plugin, {
            beat(it)
        }, 0, HEARTBEAT_INTERVAL_SEC, TimeUnit.SECONDS)
    }

    fun stop() {
        Plugin.logger.info("Stopping heart of this server...")
        shouldBeat.set(false)
    }

    private fun beat(task: ScheduledTask) {
        if (!shouldBeat.get()) {
            task.cancel()
            return
        }
        TODO("Not yet implemented")
    }
}
