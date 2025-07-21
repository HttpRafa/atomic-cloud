pluginManagement {
    includeBuild("build-logic")

    repositories {
        gradlePluginPortal()
        maven("https://repo.papermc.io/repository/maven-public/")
    }
}

plugins {
    id("org.gradle.toolchains.foojay-resolver-convention") version "1.0.0"
}

// Base
include(":api")
include(":common")

// Paper
include(":paper")
include(":paper:send")
include(":paper:notify")
include(":paper:fake-proxy")