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