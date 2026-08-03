package state.clientState.lobbyState;

import java.io.Serializable;

/**
 * Contains all the player specific lobby state, mainly the ready status
 *
 * @author Lucas Vanderwielen
 */
public class PlayerLobbyState implements Serializable {
    String playerId;
    boolean ready;

    public PlayerLobbyState(String playerId) {
        this.playerId = playerId;
        this.ready = false;
    }

    public void toggleReady() {
        this.ready = !this.ready;
    }

    public void setReady(boolean value) {
        this.ready = value;
    }

    public boolean getReady() {
        return ready;
    }

    public String getId() {
        return this.playerId;
    }
}
