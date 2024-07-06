import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    // Kotlin
    id("org.jetbrains.kotlin.jvm") version "2.0.20-Beta1"
    id("com.diffplug.spotless") version "7.0.0.BETA1"
    id("com.github.johnrengelman.shadow") version "8.1.1"
}

allprojects {
    apply(plugin = "kotlin")
    apply(plugin = "com.diffplug.spotless")
    apply(plugin = "com.github.johnrengelman.shadow")

    group = project.properties["maven_group"].toString()
    version = project.properties["client_version"].toString()

    repositories {
        mavenCentral()
    }

    tasks {
        named<ProcessResources>("processResources") {
            dependsOn("spotlessApply")
        }

        withType<KotlinCompile> {
            compilerOptions {
                jvmTarget.set(JvmTarget.JVM_21)
                freeCompilerArgs.add("-Xjvm-default=all")
            }
        }

        withType<JavaCompile> {
            options.release.set(21)
        }
    }

    // Common spotless config
    spotless {
        kotlin {
            ktlint().editorConfigOverride(mapOf("ktlint_standard_no-wildcard-imports" to "disabled"))
            trimTrailingWhitespace()
            indentWithSpaces()
        }
    }
}

tasks.named("jar") {
    dependsOn("shadowJar")
}