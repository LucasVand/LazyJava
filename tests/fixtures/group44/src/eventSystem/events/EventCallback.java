package eventSystem.events;

import java.util.function.Consumer;

// this is used internally inside of the event manager
public record EventCallback(boolean once, int id, Consumer<? extends Event> callback) {
}
