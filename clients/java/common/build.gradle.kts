plugins {
    java

    id("cloud-base")
    id("cloud-format")
    id("cloud-rpc")

    id("cloud-publish")
}

repositories {
    mavenCentral()
}

dependencies {
    implementation(project(":api"))
}