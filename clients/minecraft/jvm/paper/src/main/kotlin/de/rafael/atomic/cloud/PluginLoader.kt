package de.rafael.atomic.cloud

import io.papermc.paper.plugin.loader.PluginClasspathBuilder
import io.papermc.paper.plugin.loader.PluginLoader
import io.papermc.paper.plugin.loader.library.impl.MavenLibraryResolver
import org.eclipse.aether.artifact.DefaultArtifact
import org.eclipse.aether.graph.Dependency
import org.eclipse.aether.repository.RemoteRepository

class PluginLoader : PluginLoader {

    override fun classloader(builder: PluginClasspathBuilder) {
        val resolver = MavenLibraryResolver()
        addDependency(resolver, "org.jetbrains.kotlin:kotlin-stdlib:$KOTLIN_VERSION")
        addDependency(resolver, "io.grpc:grpc-protobuf:$GRPC_VERSION")
        addDependency(resolver, "io.grpc:grpc-stub:$GRPC_VERSION")
        addDependency(resolver, "io.grpc:grpc-netty-shaded:$GRPC_VERSION")
        addDependency(resolver, "com.google.protobuf:protobuf-java:$PROTOBUF_VERSION")
        addRepository(resolver, "https://repo.papermc.io/repository/maven-public/")
    }

    private fun addRepository(resolver: MavenLibraryResolver, url: String) {
        resolver.addRepository(RemoteRepository.Builder("paper", "default", url).build())
    }

    private fun addDependency(resolver: MavenLibraryResolver, dependency: String) {
        resolver.addDependency(Dependency(DefaultArtifact(dependency), null))
    }

    companion object {
        private const val KOTLIN_VERSION = "2.0.20-Beta1"
        private const val GRPC_VERSION = "1.65.0"
        private const val PROTOBUF_VERSION = "4.27.2"
    }
}
