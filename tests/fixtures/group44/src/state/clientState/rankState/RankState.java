package state.clientState.rankState;

import java.util.HashMap;

import eventSystem.engine.ClientEventEngine;
import state.clientState.GameData;

/**
 * RankState, the main controller of the ranking page. Contains all state that
 * is specific to the ranking page
 * 
 * @author Lucas Vanderwielen
 */
public class RankState implements GameData {
    Ranking[] rankings;
    HashMap<String, PlayerRankingState> playerState;
    ClientEventEngine client;
    String me;

    int listener;
    int restartListener;

    int nextDifficulty;
    boolean isEnd;

    boolean isRestartMode;

    Runnable onStateCallback;

    public RankState(Ranking[] rankings, PlayerRankingState[] state, int nextDifficulty, boolean isEnd, String me,
            ClientEventEngine client) {

        this.client = client;
        this.me = me;
        this.rankings = rankings;
        this.nextDifficulty = nextDifficulty;
        this.isEnd = isEnd;

        if (!this.isEnd) {
            this.listener = this.client.addListener(RankReadyEvent.class, (e) -> {
                this.handleReadyEvent(e);
            });
            this.restartListener = this.client.addListener(RankRestartToggleEvent.class, (e) -> {
                this.handleRestartToggleEvent(e);
            });
        }

        this.playerState = new HashMap<>();
        for (PlayerRankingState s : state) {
            playerState.put(s.playerId, s);
        }
        onStateCallback = () -> {
        };

    }

    /**
     * Sets the on state change callback which is called when any state is changed
     * the ui needs to rerender
     */
    @Override
    public void setOnStateChange(Runnable callback) {
        onStateCallback = callback;
    }

    /**
     * calls the on state change which updates the ui
     */
    @Override
    public void updateState() {
        onStateCallback.run();
    }

    public void drop() {
        if (!this.isEnd) {
            this.client.removeListener(this.listener);
            this.client.removeListener(this.restartListener);
        }
        onStateCallback = () -> {
        };
    }

    void handleReadyEvent(RankReadyEvent event) {
        PlayerRankingState state = playerState.get(event.playerId);

        if (state == null) {
            System.out.println("Unknown player");
            return;
        }

        state.setReady(event.state);
        updateState();
    }

    void handleRestartToggleEvent(RankRestartToggleEvent event) {
        this.isRestartMode = !this.isRestartMode;
        updateState();
    }

    /**
     * Toggles the current players ready state. Sends the events and handles the
     * state change
     */
    public void toggleReady() {
        if (this.isEnd) {
            return;
        }
        PlayerRankingState state = playerState.get(me);
        state.toggleReady();

        this.client.sendEvent(new RankReadyEvent(me, state.getReady()));
        updateState();
    }

    /**
     * Toggles the restart mode. Sends all the events and handles the state change
     */
    public void toggleRestartMode() {
        if (this.isEnd) {
            return;
        }
        this.client.sendEvent(new RankRestartToggleEvent());
        this.isRestartMode = !this.isRestartMode;
        updateState();
    }

    public boolean isRestartMode() {
        return isRestartMode;
    }

    public int getNextDifficulty() {
        return this.nextDifficulty;
    }

    public Ranking[] getRankings() {
        return rankings;
    }

    public PlayerRankingState[] getPlayerState() {
        return this.playerState.values().toArray(new PlayerRankingState[0]);
    }

    public PlayerRankingState getPlayerState(String playerId) {
        return this.playerState.get(playerId);
    }

    public PlayerRankingState getMyState() {
        return this.playerState.get(me);
    }

    public boolean isEnd() {
        return isEnd;
    }
}
