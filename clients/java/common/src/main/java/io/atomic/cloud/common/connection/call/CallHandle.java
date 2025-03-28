package io.atomic.cloud.common.connection.call;

import io.grpc.stub.ClientCallStreamObserver;
import io.grpc.stub.ClientResponseObserver;
import io.grpc.stub.StreamObserver;
import lombok.Getter;
import lombok.RequiredArgsConstructor;

@RequiredArgsConstructor
public class CallHandle<ReqT, RespT> implements ClientResponseObserver<ReqT, RespT> {

    private final StreamObserver<RespT> streamObserver;

    @Getter
    private ClientCallStreamObserver<ReqT> stream;

    public void cancel() {
        stream.cancel(null, null);
    }

    @Override
    public void beforeStart(ClientCallStreamObserver<ReqT> stream) {
        this.stream = stream;
    }

    @Override
    public void onNext(RespT value) {
        this.streamObserver.onNext(value);
    }

    @Override
    public void onError(Throwable throwable) {
        this.streamObserver.onError(throwable);
    }

    @Override
    public void onCompleted() {
        this.streamObserver.onCompleted();
    }
}
