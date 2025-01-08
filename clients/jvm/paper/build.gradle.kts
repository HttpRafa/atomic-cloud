/*buildscript {
    repositories {
        mavenCentral()
    }
    dependencies {
        classpath("com.guardsquare:proguard-gradle:7.5.0")
    }
}*/

plugins {
    // Paper
    id("io.papermc.paperweight.userdev") version "2.0.0-beta.12"

    // Shadow (Only for including the API files into the jar)
    id("com.gradleup.shadow") version "9.0.0-beta4"
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
        mergeServiceFiles()

        relocate("com.google", "io.atomic.cloud.dependencies.google")
        relocate("io.grpc", "io.atomic.cloud.dependencies.grpc")
    }

    assemble {
        dependsOn(shadowJar)
        //dependsOn("proguard")
    }

    /*
    TODO: Add in the future to reduce the size of the jar
    register<ProGuardTask>("proguard") {
        dependsOn(shadowJar)
        configuration("proguard.pro")
        verbose()

        val inputFile = shadowJar.get().archiveFile.get().asFile
        val outputFile = File(inputFile.parentFile, inputFile.nameWithoutExtension + "-min.jar")
        injars(inputFile)
        outjars(outputFile)
    }*/
}

fun getFullVersion(): String {
    val commit = System.getenv("CURRENT_COMMIT") ?: "unknown"
    val build = System.getenv("CURRENT_BUILD") ?: "0"
    return "${project.properties["client_version"]}-alpha.$commit+build.$build"
}