package state.clientState;

import java.io.Serializable;

import saveData.Stats;

/**
 * Player, this represents the general information about a player in a room.
 * This represents the current player along with any other players in the room.
 * For area specific state there is other complementary classes that are used
 * along with this to represent more attributes about the player
 *
 * @author Lucas Vanderwielen
 * 
 */
public class Player implements Serializable {
    public final String name;
    public final String id;
    private Stats stats;
    boolean connected;
    public String color;

    public final boolean isHost;

    public Player(String name, String id, boolean isHost) {
        this.name = name;
        this.id = id;
        this.color = "black";
        this.connected = true;
        this.isHost = isHost;
        this.stats = new Stats(id);
    }

    public void connect() {
        this.connected = true;
    }

    public void disconnect() {
        this.connected = false;
    }

    public Stats getStats() {
        return this.stats;
    }

    public String getName() {
        return name;
    }

    public String getId() {
        return id;
    }

    /**
     * Get whether the player is connected, the player is connected by default
     *
     * @return whether the player is connected
     */
    public boolean isConnected() {
        return connected;
    }

    public boolean isHost() {
        return isHost;
    }

    public void setColor(String color) {
        this.color = color;
    }

}
