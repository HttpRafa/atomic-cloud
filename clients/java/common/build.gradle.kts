plugins {
    java

    id("cloud-base")
    id("cloud-format")
    id("cloud-rpc")
}

repositories {
    mavenCentral()
}

dependencies {
    implementation(project(":api"))
}