package state.clientState.events;

import eventSystem.events.StandaloneEvent;
import state.clientState.typeracerState.PlayerTyperacerState;

/**
 * IntoTyperacerEvent, this is sent when moving into a typeracer page. Contains
 * the paragraph, difficulty info. along with player states an start times, and
 * points
 * 
 * @author Lucas Vanderwielen
 */
public class IntoTyperacerEvent extends StandaloneEvent {
    PlayerTyperacerState[] states;
    long startTime;
    String paragraph;
    int difficulty;

    public IntoTyperacerEvent(PlayerTyperacerState[] states, long startTime, String paragraph, int difficulty) {
        this.states = states;
        this.startTime = startTime;
        this.paragraph = paragraph;
        this.difficulty = difficulty;
    }

    public PlayerTyperacerState[] getStates() {
        return states;
    }

    public long getStartTime() {
        return startTime;
    }

    public String getParagraph() {
        return paragraph;
    }

    public int getDifficulty() {
        return difficulty;
    }

}
