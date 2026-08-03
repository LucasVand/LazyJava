package state.clientState.lobbyState;

import eventSystem.events.StandaloneEvent;

/**
 * LobbyReadyEvent, Sent when a players ready state changes. Contains the
 * updated state along with the player id
 * 
 * @author Lucas Vanderwielen
 * 
 */
public class LobbyReadyEvent extends StandaloneEvent implements LobbyEvent {
    String playerId;
    boolean readyState;

    public LobbyReadyEvent(String playerId, boolean state) {
        this.playerId = playerId;
        this.readyState = state;
    }

    public String getPlayerId() {
        return playerId;
    }

    public boolean getReadyState() {
        return readyState;
    }
}
