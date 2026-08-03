package state.clientState.events;

import eventSystem.events.StandaloneEvent;
import state.clientState.rankState.PlayerRankingState;
import state.clientState.rankState.Ranking;

/**
 * IntoRankingEvent, sent when leaving the typeracer. Contains all the info
 * about the ranking page, player rankings, next difficulty and whether it is
 * the end
 * 
 * @author Lucas Vanderwielen
 */
public class IntoRankingEvent extends StandaloneEvent {
    Ranking[] rankings;
    PlayerRankingState[] state;
    int nextDifficulty;
    boolean isEnd;

    public IntoRankingEvent(Ranking[] rankings, PlayerRankingState[] state, int nextDifficulty, boolean isEnd) {
        this.rankings = rankings;
        this.state = state;
        this.nextDifficulty = nextDifficulty;
        this.isEnd = isEnd;
    }

    public Ranking[] getRankings() {
        return rankings;
    }

    public PlayerRankingState[] getState() {
        return state;
    }

    public int getNextDifficulty() {
        return nextDifficulty;
    }

    public boolean isEnd() {
        return this.isEnd;
    }
}
