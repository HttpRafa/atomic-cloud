package io.atomic.cloud.api;

import io.atomic.cloud.api.channel.Channels;
import io.atomic.cloud.api.objects.LocalCloudServer;
import io.atomic.cloud.api.transfer.Transfers;

public class Cloud {

    private static CloudAPI INSTANCE;

    /**
     * Set up the Cloud API | Do not call this method
     * @param instance the Cloud API instance
     */
    public static void setup(CloudAPI instance) {
        if (Cloud.INSTANCE != null) throw new IllegalStateException();
        Cloud.INSTANCE = instance;
    }

    /**
     * Get the current server instance
     * @return the current server instance
     */
    public static LocalCloudServer self() {
        return Cloud.INSTANCE.self();
    }

    /**
     * The channels API
     * @return the channels API
     */
    public static Channels channels() {
        return Cloud.INSTANCE.channels();
    }

    /**
     * The transfer API
     * @return the transfer API
     */
    public static Transfers transfers() {
        return Cloud.INSTANCE.transfers();
    }

    /**
     * The server marks itself ready when it is started. This method disables this behavior.
     * This is useful if you want to control when the server is ready yourself.
     */
    public static void disableAutoReady() {
        Cloud.INSTANCE.disableAutoReady();
    }

    public interface CloudAPI {
        /**
         * Get the current server instance
         * @return the current server instance
         */
        LocalCloudServer self();

        /**
         * The channels API
         * @return the channels API
         */
        Channels channels();

        /**
         * The transfer API
         * @return the transfer API
         */
        Transfers transfers();

        /**
         * The server marks itself ready when it is started. This method disables this behavior.
         * This is useful if you want to control when the server is ready yourself.
         */
        void disableAutoReady();
    }
}
