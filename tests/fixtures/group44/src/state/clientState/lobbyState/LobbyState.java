package state.clientState.lobbyState;

import java.util.HashMap;

import eventSystem.engine.ClientEventEngine;
import state.clientState.GameData;

/**
 * LobbyState, the main controller of the lobby page. Contains all state
 * specific to the lobby page.
 * 
 * @author Lucas Vanderwielen
 */
public class LobbyState implements GameData {

    int readyEventListener;
    ClientEventEngine client;
    HashMap<String, PlayerLobbyState> playerState;
    String me;

    Runnable onStateChange;

    public LobbyState(PlayerLobbyState[] state, String me, ClientEventEngine client) {

        this.me = me;

        this.playerState = new HashMap<>();
        for (PlayerLobbyState s : state) {
            this.playerState.put(s.playerId, s);
        }

        this.client = client;

        this.readyEventListener = client.addListener(LobbyReadyEvent.class, (e) -> {
            this.handleReadyEvent(e);
        });

        this.onStateChange = () -> {
        };

    }

    @Override
    public void setOnStateChange(Runnable callback) {
        this.onStateChange = callback;
    }

    @Override
    public void updateState() {
        onStateChange.run();
    }

    public void drop() {
        this.client.removeListener(this.readyEventListener);
        this.onStateChange = () -> {
        };
    }

    /** Toggles the current players ready state */
    public void toggleReady() {
        PlayerLobbyState state = playerState.get(me);
        state.toggleReady();
        this.client.sendEvent(new LobbyReadyEvent(me, state.ready));

        this.updateState();
    }

    public PlayerLobbyState[] getPlayerState() {
        return playerState.values().toArray(new PlayerLobbyState[0]);
    }

    public PlayerLobbyState getPlayerState(String playerId) {
        return this.playerState.get(playerId);
    }

    public void handleJoinEvent(LobbyJoinEvent event) {
        PlayerLobbyState state = new PlayerLobbyState(event.playerId);

        this.playerState.put(event.playerId, state);
        this.updateState();
    }

    void handleReadyEvent(LobbyReadyEvent event) {
        PlayerLobbyState state = this.playerState.get(event.playerId);
        if (state == null) {
            System.out.println("Unknown playerId");
            return;
        }
        state.setReady(event.readyState);
        this.updateState();
    }

}
