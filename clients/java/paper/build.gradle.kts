plugins {
    java

    id("cloud-base")
    id("cloud-format")
    id("cloud-rpc")

    id("cloud-publish")

    id("io.papermc.paperweight.userdev") version "2.0.0-beta.16"

    // Shadow (Only for including the API files into the jar)
    id("com.gradleup.shadow") version "9.0.0-beta11"
}

repositories {
    mavenCentral()
}

dependencies {
    // Paper and NMS
    paperweight.paperDevBundle("${project.property("minecraft.version")}-R0.1-SNAPSHOT")

    // The cloud dependencies
    implementation(project(":api"))
    implementation(project(":common"))
}

tasks {
    processResources {
        filesMatching("paper-plugin.yml") {
            expand("client_version" to getFullVersion())
        }
    }

    assemble {
        dependsOn(shadowJar)
    }
}

fun getFullVersion(): String {
    val commit = System.getenv("CURRENT_COMMIT") ?: "unknown"
    val build = System.getenv("CURRENT_BUILD") ?: "0"
    return "${project.property("project.version")}.$commit+build.$build"
}