package de.rafael.atomic.cloud

import de.rafael.atomic.cloud.api.CloudAPI

object Cloud {

    @JvmStatic
    private var api: CloudAPI? = null

    fun setup(api: CloudAPI) {
        if (this.api != null) throw IllegalStateException()
        this.api = api
    }
}
