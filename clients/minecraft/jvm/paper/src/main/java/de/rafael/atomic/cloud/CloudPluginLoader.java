package de.rafael.atomic.cloud;

import io.papermc.paper.plugin.loader.PluginClasspathBuilder;
import io.papermc.paper.plugin.loader.PluginLoader;
import io.papermc.paper.plugin.loader.library.impl.MavenLibraryResolver;
import org.eclipse.aether.artifact.DefaultArtifact;
import org.eclipse.aether.graph.Dependency;
import org.eclipse.aether.repository.RemoteRepository;
import org.jetbrains.annotations.ApiStatus;
import org.jetbrains.annotations.NotNull;

@ApiStatus.Experimental
public class CloudPluginLoader implements PluginLoader {

    private static final String GRPC_VERSION = "1.65.0";
    private static final String PROTOBUF_VERSION = "4.27.2";

    @Override
    public void classloader(@NotNull PluginClasspathBuilder builder) {
        var resolver = new MavenLibraryResolver();
        addDependency(resolver, "io.grpc:grpc-protobuf:" + GRPC_VERSION);
        addDependency(resolver, "io.grpc:grpc-stub:" + GRPC_VERSION);
        addDependency(resolver, "io.grpc:grpc-netty-shaded:" + GRPC_VERSION);
        addDependency(resolver, "com.google.protobuf:protobuf-java:" + PROTOBUF_VERSION);
        addRepository(resolver, "https://repo.papermc.io/repository/maven-public/");

        builder.addLibrary(resolver);
    }

    private void addRepository(@NotNull MavenLibraryResolver resolver, String url) {
        resolver.addRepository(new RemoteRepository.Builder("paper", "default", url).build());
    }

    private void addDependency(@NotNull MavenLibraryResolver resolver, String dependency) {
        resolver.addDependency(new Dependency(new DefaultArtifact(dependency), null));
    }
}
