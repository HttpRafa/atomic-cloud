import com.google.protobuf.gradle.id

plugins {
    java

    id("com.google.protobuf")
}

dependencies {
    compileOnly("io.grpc:grpc-protobuf:${project.property("grpc.version")}")
    compileOnly("io.grpc:grpc-stub:${project.property("grpc.version")}")
    compileOnly("com.google.protobuf:protobuf-java:${project.property("protobuf.version")}")
}

sourceSets {
    main {
        proto {
            srcDir("$rootDir/../../protocol/grpc/")
        }
    }
}

protobuf {
    protoc {
        artifact = "com.google.protobuf:protoc:${project.property("protobuf.version")}"
    }
    plugins {
        id("grpc") {
            artifact = "io.grpc:protoc-gen-grpc-java:${project.property("grpc.version")}"
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