import com.google.protobuf.gradle.id

plugins {
    id("com.google.protobuf") version "0.9.4"
}

dependencies {
    compileOnly("io.grpc:grpc-protobuf:${project.properties["grpc_version"]}")
    compileOnly("io.grpc:grpc-stub:${project.properties["grpc_version"]}")
    compileOnly("com.google.protobuf:protobuf-java:${project.properties["protobuf_version"]}")
    runtimeOnly("io.grpc:grpc-netty-shaded:${project.properties["grpc_version"]}")

    // The cloud API
    compileOnly(project(":api"))
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
            srcDir("$rootDir/../../../protocol/grpc/")
        }
    }
}