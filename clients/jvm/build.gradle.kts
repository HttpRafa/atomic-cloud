import com.google.protobuf.gradle.id

plugins {
    id("java")
    id("com.diffplug.spotless") version "7.0.0.BETA1"
    id("com.google.protobuf") version "0.9.4"
}

allprojects {
    apply(plugin = "java")
    apply(plugin = "com.diffplug.spotless")
    apply(plugin = "com.google.protobuf")

    group = project.properties["maven_group"].toString()
    version = "${project.properties["client_version"]}-SNAPSHOT"

    repositories {
        mavenCentral()
    }

    dependencies {
        // gRPC
        compileOnly("io.grpc:grpc-protobuf:${project.properties["grpc_version"]}")
        compileOnly("io.grpc:grpc-stub:${project.properties["grpc_version"]}")
        compileOnly("com.google.protobuf:protobuf-java:${project.properties["protobuf_version"]}")
        runtimeOnly("io.grpc:grpc-netty-shaded:${project.properties["grpc_version"]}")

        // Jetbrains annotations
        compileOnly("org.jetbrains:annotations:${project.properties["jetbrains_annotations_version"]}")

        // Lombok
        compileOnly("org.projectlombok:lombok:${project.properties["lombok_version"]}")
        annotationProcessor("org.projectlombok:lombok:${project.properties["lombok_version"]}")
    }

    tasks {
        named<ProcessResources>("processResources") {
            dependsOn("spotlessApply")
        }

        withType<JavaCompile> {
            options.release.set(21)
        }
    }

    // Common spotless config
    spotless {
        java {
            trimTrailingWhitespace()
            indentWithSpaces()
            removeUnusedImports()
            palantirJavaFormat()
        }
    }

    protobuf {
        protoc {
            artifact = "com.google.protobuf:protoc:${project.properties["protobuf_version"]}"
        }
        plugins {
            id("grpc") {
                artifact = "io.grpc:protoc-gen-grpc-java:${project.properties["grpc_version"]}"
            }
        }
        generateProtoTasks {
            all().forEach { task ->
                task.plugins {
                    id("grpc") {
                        option("@generated=omit")
                    }
                }
            }
        }
    }

    sourceSets {
        main {
            proto {
                srcDir("$rootDir/../../protocol/grpc/server/")
            }
        }
    }
}