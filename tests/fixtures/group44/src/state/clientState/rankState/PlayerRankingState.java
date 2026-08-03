package state.clientState.rankState;

import java.io.Serializable;

/**
 * contains all the state specific to the ranking page. Contains information
 * about whether the player is ready
 * 
 * @author Lucas Vanderwielen
 */
public class PlayerRankingState implements Serializable {
    String playerId;
    boolean ready;

    public PlayerRankingState(String playerId) {
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
        return playerId;
    }

}
