package state.clientState.rankState;

import java.io.Serializable;

/**
 * Ranking, this is a data object that contains info about the previous games
 * ranking. Contains the playerId, progress in the paragraph, time taken to
 * complete ({@code Long.MAX_VALUE} if DNF), the peak wpm, and the number of
 * points earned that game
 */
public record Ranking(String playerId, double progress, long time, boolean completed, double peakWPM, int points)
        implements Serializable {

}
