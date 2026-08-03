package state.clientState.typeracerState;

import eventSystem.events.StandaloneEvent;
import state.clientState.typeracerState.PlayerTyperacerState.PlayerStatus;

/**
 * TyperacerStatusEvent, this event is sent when the status of a player changes.
 * That could be when they run out of lives or complete the paragraph. Contains
 * the playerId and the updated state
 * 
 * @author Lucas Vanderwielen
 */
public class TyperacerStatusEvent extends StandaloneEvent {

    String playerId;
    PlayerStatus status;

    public TyperacerStatusEvent(String id, PlayerStatus status) {
        this.playerId = id;
        this.status = status;
    }

    public String getId() {
        return this.playerId;
    }

    public PlayerStatus getStatus() {
        return this.status;
    }
}
