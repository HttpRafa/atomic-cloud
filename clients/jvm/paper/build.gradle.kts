plugins {
    // Paper
    id("io.papermc.paperweight.userdev") version "1.7.1"

    // Shadow (Only for including the API files into the jar)
    id("com.github.johnrengelman.shadow") version "8.1.1"
}

dependencies {
    // Paper version x
    paperweight.paperDevBundle("${project.properties["minecraft_version"]}-R0.1-SNAPSHOT")

    // The cloud API
    implementation(project(":api"))
    implementation(project(":common"))
}

tasks {
    processResources {
        val fullVersion = getFullVersion()
        inputs.properties(mapOf("client_version" to fullVersion))

        filesMatching("paper-plugin.yml") {
            expand(mapOf("client_version" to fullVersion))
        }
    }

    shadowJar {
        dependencies {
            exclude(".*:.*")
            include(project(":api"))
            include(project(":common"))
        }
    }

    assemble {
        dependsOn(shadowJar)
    }
}

fun getFullVersion(): String {
    val commit = System.getenv("CURRENT_COMMIT") ?: "unknown"
    val build = System.getenv("CURRENT_BUILD") ?: "0"
    return "${project.properties["client_version"]}-alpha.$commit+build.$build"
}