package eventSystem.networking.server;

import eventSystem.networking.RawEvent;

/// A traceable wrapper over a {@code RawEvent}, allows the server to know which
/// client the event came from, this is needed for {@code ResponseEvent} handling
public record TraceableRawEvent(Integer clientId, RawEvent event) {
}
