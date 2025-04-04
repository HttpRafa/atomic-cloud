package io.atomic.cloud.paper;

import io.papermc.paper.plugin.loader.PluginClasspathBuilder;
import io.papermc.paper.plugin.loader.PluginLoader;
import io.papermc.paper.plugin.loader.library.impl.MavenLibraryResolver;
import org.eclipse.aether.artifact.DefaultArtifact;
import org.eclipse.aether.graph.Dependency;
import org.eclipse.aether.repository.RemoteRepository;
import org.jetbrains.annotations.NotNull;

@SuppressWarnings("UnstableApiUsage")
public class CloudPluginLoader implements PluginLoader {

    private static final String GRPC_VERSION = "1.69.0";

    @Override
    public void classloader(@NotNull PluginClasspathBuilder builder) {
        MavenLibraryResolver resolver = new MavenLibraryResolver();
        resolver.addRepository(
                new RemoteRepository.Builder("paper", "default", "https://repo1.maven.org/maven2/").build());
        dependency(resolver, "io.grpc", "grpc-protobuf", GRPC_VERSION);
        dependency(resolver, "io.grpc", "grpc-stub", GRPC_VERSION);
        dependency(resolver, "io.grpc", "grpc-netty", GRPC_VERSION);

        builder.addLibrary(resolver);
    }

    private static void dependency(
            @NotNull MavenLibraryResolver resolver, String groupId, String artifactId, String version) {
        resolver.addDependency(new Dependency(new DefaultArtifact(groupId + ":" + artifactId + ":" + version), null));
    }
}
