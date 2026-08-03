package state.clientState.lobbyState;

import eventSystem.events.StandaloneEvent;
import state.clientState.Player;

/**
 * LobbyJoinEvent, Send when a player has joined the room. Requires the player
 * to be added to the state. Contains the playerId and the player object
 * 
 * @author Lucas Vanderwielen
 */
public class LobbyJoinEvent extends StandaloneEvent implements LobbyEvent {
    String playerId;
    Player player;

    public LobbyJoinEvent(String playerId, Player player) {
        this.player = player;
        this.playerId = playerId;
    }

    public Player getPlayer() {
        return player;
    }

    public String getPlayerId() {
        return playerId;
    }
}
