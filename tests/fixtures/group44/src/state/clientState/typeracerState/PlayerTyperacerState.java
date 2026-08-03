package state.clientState.typeracerState;

import java.io.Serializable;

/**
 * PlayerTyperacerState, represents the players attributes in the typeracer
 * setting. This represents the current player and any other players in the
 * room.
 * 
 * @author Lucas Vanderwielen
 */
public class PlayerTyperacerState implements Serializable {
    String playerId;
    int position;
    PlayerStatus status;

    int lives;

    public PlayerTyperacerState(String id) {
        this.playerId = id;
        this.position = 0;
        this.status = PlayerStatus.Playing;
        this.lives = 150;
    }

    /**
     * Gets the players id, this is a helper function to be used alongside
     * other functions that require the player id as a param
     */
    public String getPlayerId() {
        return playerId;
    }

    /**
     * Gets the players position in the paragraph, this is updated
     * automatically when the {@code keyTyped} function is called or from an event
     * from the server, the units are characters
     */
    public int getPosition() {
        return position;
    }

    /**
     * Sets the players position
     * 
     * @param position the new position to set
     */
    public void setPosition(int position) {
        this.position = position;
    }

    /**
     * Subtracts from the players lives and sets the status if the player runs out
     * of lives
     */
    public void loseLife() {
        this.lives -= 10;

        if (this.lives <= 0) {
            this.lives = 0;
            this.status = PlayerStatus.Dead;
        }
    }

    /**
     * Get the player current status
     *
     * @return the players status
     */
    public PlayerStatus getStatus() {
        return status;
    }

    public int getLives() {
        return lives;
    }

    public static enum PlayerStatus {
        Completed, Dead, Playing
    }

    public void setStatus(PlayerStatus status) {
        this.status = status;
    }

    public void setLives(int lives) {
        this.lives = lives;
    }
}
