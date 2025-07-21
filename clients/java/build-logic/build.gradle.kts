plugins {
    `kotlin-dsl`
}

repositories {
    gradlePluginPortal()
}

dependencies {
    implementation("com.google.protobuf:protobuf-gradle-plugin:0.9.5")
    implementation("com.diffplug.spotless:spotless-plugin-gradle:7.2.0")
}