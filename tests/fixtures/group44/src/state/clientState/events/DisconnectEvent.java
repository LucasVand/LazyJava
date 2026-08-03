package state.clientState.events;

import eventSystem.events.StandaloneEvent;

/**
 * DisconnectEvent
 *
 * @author Lucas Vanderwielen
 */
public class DisconnectEvent extends StandaloneEvent {
    String id;

    public DisconnectEvent(String id) {
        this.id = id;
    }

    public String getId() {
        return id;
    }
}
