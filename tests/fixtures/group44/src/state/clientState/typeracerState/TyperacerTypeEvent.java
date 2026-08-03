package state.clientState.typeracerState;

import eventSystem.events.StandaloneEvent;

/**
 * TyperacerTypeEvent, sent when a player types a key and moves positions along
 * the paragraph. Contains updates stats for the player along with the new
 * position
 * 
 * @author Lucas Vanderwielen
 */
public class TyperacerTypeEvent extends StandaloneEvent {
    String playerId;
    int position;
    double peakWPM;
    int points;

    public TyperacerTypeEvent(String playerId, int pos, double peakWPM, int points) {
        this.position = pos;
        this.playerId = playerId;
        this.peakWPM = peakWPM;
        this.points = points;
    }

    public String getPlayerId() {
        return playerId;
    }

    public int getPosition() {
        return position;
    }

    public double getPeakWPM() {
        return peakWPM;
    }

    public int getPoints() {
        return points;
    }

}
