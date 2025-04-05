package io.atomic.cloud.common.connection.credential;

import io.grpc.CallCredentials;
import io.grpc.Metadata;
import java.util.concurrent.Executor;
import lombok.RequiredArgsConstructor;
import org.jetbrains.annotations.NotNull;

@RequiredArgsConstructor
public class TokenCredential extends CallCredentials {

    private final String token;

    @Override
    public void applyRequestMetadata(RequestInfo requestInfo, Executor executor, @NotNull MetadataApplier applier) {
        var metadata = new Metadata();
        metadata.put(Metadata.Key.of("authorization", Metadata.ASCII_STRING_MARSHALLER), this.token);
        applier.apply(metadata);
    }
}
