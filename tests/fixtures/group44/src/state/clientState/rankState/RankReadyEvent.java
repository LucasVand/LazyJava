package state.clientState.rankState;

import eventSystem.events.StandaloneEvent;

/**
 * RankReadyEvent, sent when a player ready state changes in on the ranking
 * page, contain info about the new state and the playerId
 * 
 * @author Lucas Vanderwielen
 */
public class RankReadyEvent extends StandaloneEvent {
    String playerId;
    boolean state;

    public RankReadyEvent(String playerId, boolean state) {
        this.playerId = playerId;
        this.state = state;
    }

    public String getPlayerId() {
        return playerId;
    }

    public boolean getReady() {
        return state;
    }

}
