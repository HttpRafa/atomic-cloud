import gradle.kotlin.dsl.accessors._9994178b0534ad6129be0f302527175d.java

plugins {
    java
}

group = project.property("maven.group").toString();
version = project.property("project.version").toString();

repositories {
    mavenCentral()
}

dependencies {
    // JetBrains annotations
    compileOnly("org.jetbrains:annotations:${project.property("jetbrains.annotations.version")}")

    // Lombok
    compileOnly("org.projectlombok:lombok:${project.property("lombok.version")}")
    annotationProcessor("org.projectlombok:lombok:${project.property("lombok.version")}")
}

java {
    sourceCompatibility = JavaVersion.VERSION_21
    targetCompatibility = JavaVersion.VERSION_21
    // Enable sources jar
    withSourcesJar()

    toolchain {
        languageVersion = JavaLanguageVersion.of(21)
    }
}

tasks {
    compileJava {
        options.encoding = "UTF-8"
    }
}