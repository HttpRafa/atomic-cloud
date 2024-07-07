package de.rafael.atomic.cloud.api

object Cloud {

    @JvmStatic
    private var api: CloudInterface? = null

    fun setup(api: CloudInterface) {
        if (Cloud.api != null) throw IllegalStateException()
        Cloud.api = api
    }

    interface CloudInterface
}
