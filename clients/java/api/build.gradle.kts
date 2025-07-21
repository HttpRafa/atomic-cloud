plugins {
    java

    id("cloud-base")
    id("cloud-format")
    id("cloud-rpc")

    id("cloud-publish")

    // Shadow (Only for including the API files into the jar)
    id("com.gradleup.shadow") version "9.0.0-rc1"
}

tasks {
    assemble {
        dependsOn(shadowJar)
    }
}