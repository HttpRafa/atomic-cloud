package de.rafael.atomic.cloud.heart

import de.rafael.atomic.cloud.Plugin
import io.papermc.paper.threadedregions.scheduler.ScheduledTask
import org.bukkit.Bukkit
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import java.util.function.Consumer

object Heart : Consumer<ScheduledTask> {

    private const val HEARTBEAT_INTERVAL_SEC = 10L // Every 10 seconds
    private val shouldBeat = AtomicBoolean(true)

    fun start() {
        Plugin.logger.info("Starting heart of this server...")
        shouldBeat.set(true)
        Bukkit.getAsyncScheduler().runAtFixedRate(Plugin, Heart, 0, HEARTBEAT_INTERVAL_SEC, TimeUnit.SECONDS)
    }

    fun stop() {
        Plugin.logger.info("Stopping heart of this server...")
        shouldBeat.set(false)
    }

    override fun accept(task: ScheduledTask) {
        if (!shouldBeat.get()) {
            task.cancel()
            return
        }
        TODO("Not yet implemented")
    }
}
