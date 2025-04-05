package io.atomic.cloud.common.cache;

import java.util.Optional;

public class CachedObject<T> {

    private static final long DEFAULT_EXPIRATION = 1000 * 30;

    private T value;
    private long invalidateTime;

    public synchronized Optional<T> getValue() {
        if (this.value == null) return Optional.empty();
        if (System.currentTimeMillis() > invalidateTime) {
            this.value = null;
            return Optional.empty();
        }
        return Optional.of(this.value);
    }

    public synchronized void setValue(T value) {
        this.value = value;
        this.invalidateTime = System.currentTimeMillis() + DEFAULT_EXPIRATION;
    }
}
