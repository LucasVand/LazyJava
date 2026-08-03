package state.clientState.events;

import eventSystem.events.ResponseEvent;
import eventSystem.events.StandaloneEvent;
import state.clientState.Player;
import state.clientState.lobbyState.PlayerLobbyState;

/**
 * JoinEvent
 * 
 * @author Lucas Vanderwielen
 */
public class JoinEvent extends ResponseEvent<JoinEvent.JoinEventRes> {
    Player player;

    public JoinEvent(Player me) {
        this.player = me;
    }

    public Player getPlayer() {
        return player;
    }

    public static class JoinEventRes extends StandaloneEvent {
        Player[] playerList;
        PlayerLobbyState[] lobbyStates;

        public JoinEventRes(Player[] list, PlayerLobbyState[] lobby) {
            this.playerList = list;
            this.lobbyStates = lobby;
        }

        public Player[] getPlayerList() {
            return this.playerList;
        }

        public PlayerLobbyState[] getLobbyList() {
            return this.lobbyStates;
        }
    }
}
