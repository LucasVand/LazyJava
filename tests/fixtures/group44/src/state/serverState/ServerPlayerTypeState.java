package state.serverState;

/**
 * ServerPlayerTypeState
 * 
 * @author Lucas Vanderwielen
 */
public class ServerPlayerTypeState {
    String playerId;
    boolean completed;
    long completedTime;
    double peakWPM;
    int points;

    public ServerPlayerTypeState(String id) {
        this.playerId = id;
        this.completed = false;
        this.completedTime = Long.MAX_VALUE;
        this.peakWPM = 0.0;
        this.points = 0;
    }

    public void complete(long time) {
        this.completed = true;
        this.completedTime = time;
    }
}
