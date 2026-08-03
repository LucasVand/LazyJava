package state.clientState.typeracerState;

import java.util.ArrayList;
import java.util.HashMap;

import javax.swing.Timer;

import eventSystem.engine.ClientEventEngine;
import saveData.SaveData;
import saveData.Stats;
import state.clientState.GameData;
import state.clientState.Powerups;
import state.clientState.typeracerState.PlayerTyperacerState.PlayerStatus;

/**
 * TyperacerState. This is the main controller when in the typeracer state.
 * Handles all of the event sending and state updates via event listeners. This
 * class should not be created manually, it is mangaged by the
 * {@code ClientState} class and be accessed via that
 * 
 * @author Lucas Vanderwielen
 */
public class TyperacerState implements GameData {
    public static final long DURATION = 1000 * 240;

    // the state of all the players in the game
    HashMap<String, PlayerTyperacerState> playerState;
    // current state of the game
    State state;

    String paragraph;
    // start time of the game
    long startTime;
    // end time of the game
    long endTime;
    // the id of the current logged in player
    String me;
    // the engine for handling events
    ClientEventEngine client;
    // the state of the chars already typed
    ArrayList<CharState> charStates;

    // the stats for the current game
    Stats stats;
    // current games difficulty
    int difficulty;
    // the points of the current logged in player
    int points;

    // a callback that is called when any state changes and requires a ui update
    Runnable onStateChange = () -> {
    };

    int keyTypedEvent;
    int statusEvent;

    public double multiplier = 1.0;
    public int streak = 0;
    public boolean isBoostActive = false;

    public Powerups powerups;

    public TyperacerState(PlayerTyperacerState[] playerState, String paragraph, long startTime, int difficulty,
            String me,
            ClientEventEngine client, Powerups powerups) {

        this.points = 0;
        this.me = me;
        this.client = client;
        this.startTime = startTime;
        this.endTime = startTime + TyperacerState.DURATION;
        this.paragraph = paragraph;
        this.playerState = new HashMap<>();
        this.state = State.Countdown;
        this.stats = new Stats(SaveData.getData().getLoggedInAccount().getId());
        this.stats.setDifficulty(difficulty);

        this.difficulty = difficulty;

        this.powerups = powerups;

        for (PlayerTyperacerState p : playerState) {
            this.playerState.put(p.playerId, p);
        }

        keyTypedEvent = this.client.addListener(TyperacerTypeEvent.class, (e) -> {
            this.handleKeyTypedEvent(e);
        });
        statusEvent = this.client.addListener(TyperacerStatusEvent.class, (e) -> {
            this.handleStatusEvent(e);
        });

        this.charStates = new ArrayList<>(paragraph.length());

        int delay = (int) (this.startTime - System.currentTimeMillis());
        Timer t = new Timer(delay, (e) -> {
            startPlayingState();
        });
        t.setRepeats(false);
        t.start();
    }

    /**
     * Sets the on state change callback, this callback is called whenever any state
     * changes which requires a ui update
     *
     * @param r the callback which is to be called
     */
    public void setOnStateChange(Runnable r) {
        this.onStateChange = r;
    }

    /**
     * Calls the onStateChange callback which rerenders the ui
     */
    public void updateState() {
        onStateChange.run();
    }

    void handleKeyTypedEvent(TyperacerTypeEvent e) {
        PlayerTyperacerState p = playerState.get(e.getPlayerId());

        if (p != null) {
            p.setPosition(e.position);
        }
        this.updateState();
    }

    void handleStatusEvent(TyperacerStatusEvent e) {
        PlayerTyperacerState p = playerState.get(e.getId());

        p.setStatus(e.getStatus());

        this.updateState();
    }

    void startPlayingState() {
        state = TyperacerState.State.Playing;
    }

    /**
     * Get the amount of time left in the count down, units milliseconds. Will
     * display
     * negative numbers
     * when the countdown is over.
     */
    public long countDownTime() {
        return this.startTime - System.currentTimeMillis();
    }

    /**
     * Gets the amount of time remaining in the game in milliseconds. value is zero
     * if the game has not started and the countdown is still running. Will go
     * negative if past the end time.
     */
    public long timeRemaining() {
        if (countDownTime() > 0) {
            return TyperacerState.DURATION;
        }
        return this.endTime - System.currentTimeMillis();
    }

    /**
     * Gets the state of the already typed characters. The state only exists for
     * characters that have already been typed. So the size is less then the current
     * position of the player
     */
    public ArrayList<CharState> getCharState() {
        return charStates;
    }

    /**
     * Get the paragraph that is to be typed
     */
    public String getParagraph() {
        return this.paragraph;
    }

    public State getState() {
        return this.state;
    }

    public int getDifficulty() {
        return this.difficulty;
    }

    public PlayerTyperacerState getMyState() {
        return this.playerState.get(me);
    }

    /**
     * Get the state of all the players in the typeracer
     */
    public PlayerTyperacerState[] getPlayerState() {
        return this.playerState.values().toArray(new PlayerTyperacerState[0]);
    }

    /**
     * Gets the state of the given playerId, if the id does not exist the function
     * returns null
     *
     * @param playerId the id which to get the state from
     */
    public PlayerTyperacerState getPlayerState(String playerId) {
        return this.playerState.get(playerId);
    }

    /**
     * Updates the state and send the appropriate events to the server for a key
     * press. should be called when the user types a key
     */
    public void keyTyped(char ch) {
        keyTypedInternal(ch, true, true);

    }

    /**
     * Internal handler for processing a single keystroke during the typing race.
     * Evaluates character accuracy, manages streaks and multipliers, handles
     * powerup
     * activation via number keys, updates player health, and broadcasts events to
     * the server.
     *
     * @param ch          The character typed by the user.
     * @param sendEvent   If true, broadcasts the player's updated state
     *                    (position/points) to the network.
     * @param updateStats If true, logs the keystroke to calculate WPM and overall
     *                    accuracy.
     */
    private void keyTypedInternal(char ch, boolean sendEvent, boolean updateStats) {
        // Guard clause: Only process keystrokes if the game is active and the player is
        // alive
        if (this.state != State.Playing || getMyState().status != PlayerStatus.Playing) {
            return;
        }

        // --- Powerup Handling ---
        if (Character.isDigit(ch)) {
            // ADD CHECKS TO mAKE SURE THEY AVAILABLE
            // Map keys '1', '2', and '3' to their respective powerups, ensuring charges
            // exist
            if (ch == '1' && powerups.getBoosts().getCharges() != 0)
                activateBoost();
            else if (ch == '2' && powerups.getHearts().getCharges() != 0)
                activateHeart();
            else if (ch == '3' && powerups.getSkips().getCharges() != 0)
                activateSkip();

            // Refresh the UI to reflect the consumed powerup charge
            updateState();
            return;
        }

        // --- Typing Logic ---
        PlayerTyperacerState state = getMyState();

        // Determine the actual character the player is supposed to type right now
        char current = paragraph.charAt(state.position);

        // Advance the player's position in the text and award flat base points for
        // typing
        state.position += 1;
        points += 2;

        // Log the keystroke for WPM and accuracy metrics if requested
        if (updateStats) {
            stats.typeChar(ch, current);
        }

        // --- Accuracy Evaluation ---
        if (current == ch) {
            // The player typed the correct character
            charStates.add(CharState.Correct);

            // Calculate points based on the current streak multiplier
            int pts = (int) (10 * multiplier);
            // Double the earned points if a boost powerup is currently active
            if (isBoostActive)
                pts *= 2;

            points += pts;

            // Increment the consecutive correct keystroke streak
            streak++;
            // Every 10 correct strokes in a row, increase the point multiplier by 0.1
            if (streak % 10 == 0)
                multiplier += 0.1;
        } else {
            // The player typed the wrong character
            charStates.add(CharState.Incorrect);

            // Penalize the player by removing 10 health/lives
            state.setLives(state.getLives() - 10);

            // Break the streak and reset the point multiplier back to base (1.0x)
            streak = 0;
            multiplier = 1.0;

            // If the health penalty drops them to 0 or below, trigger the death state
            if (state.getLives() <= 0) {
                die();
            }
        }

        // --- Network & Completion ---

        // Broadcast the new position, WPM, and score to the server to update opponent
        // screens
        if (sendEvent) {
            this.client.sendEvent(new TyperacerTypeEvent(me, state.position, stats.getPeakWPM(), points));
        }

        // Check if the player has successfully typed the very last character of the
        // paragraph
        if (state.position == paragraph.length()) {
            complete();
        }

        // Ensure all state changes are pushed to the UI
        updateState();
    }

    /**
     * Activates the "Boost" powerup. This applies a 2x point multiplier to all
     * correct keystrokes for a duration of 5 seconds. Consumes one boost charge.
     */
    void activateBoost() {
        // Only allow activation if a boost isn't already running to prevent overlap
        if (!isBoostActive) {
            isBoostActive = true;
            // might alr exist

            // Create a timer to automatically turn the boost off after 5000 milliseconds (5
            // seconds)
            Timer t = new Timer(5000, e -> isBoostActive = false);
            // Ensure the timer only fires once, rather than repeating every 5 seconds
            t.setRepeats(false);
            t.start();

            // Deduct the powerup from the player's inventory
            powerups.getBoosts().removeCharge();
        }
    }

    void die() {
        getMyState().setStatus(PlayerStatus.Dead);
        client.sendEvent(new TyperacerStatusEvent(me, getMyState().getStatus()));
    }

    /**
     * Activates the "Skip" powerup. This instantly auto-completes the current word
     * the player is typing by simulating the correct keystrokes up to the next
     * space
     * character. Consumes one skip charge.
     */
    void activateSkip() {
        PlayerTyperacerState state = getMyState();

        // Only allow the powerup to be used if the player is currently alive and active
        if (state.getStatus() == PlayerStatus.Playing) {
            // Deduct the powerup from the player's inventory
            powerups.getSkips().removeCharge();

            // Find the character the player is currently supposed to type
            char nextChar = paragraph.charAt(state.getPosition());

            // Loop through the paragraph until we hit a space (the end of the current word)
            while (nextChar != ' ') {
                // Simulate typing the correct character.
                // sendEvent = false: Don't spam the server with an update for every single
                // letter.
                // updateStats = false: Don't artificially inflate the player's WPM or accuracy.
                keyTypedInternal(nextChar, false, false);

                // Fetch the next character in the sequence
                nextChar = paragraph.charAt(state.getPosition());
            }

            // Type the space character to officially finish the word.
            // sendEvent = true: Now we broadcast the massive position jump to the server so
            // other players see it.
            keyTypedInternal(nextChar, true, false);
        }
    }

    /**
     * Activates the "Heart" (health) powerup.
     * This restores 20 health points to the player, provided they are still alive
     * and have not already reached the maximum health limit of 150. Consumes one
     * heart charge.
     */
    void activateHeart() {
        // Retrieve the current state and health metrics of the local player
        PlayerTyperacerState state = getMyState();

        // Only allow healing if the player is currently alive (lives > 0)
        // and hasn't reached the maximum health cap (150)
        if (state.getLives() > 0 && state.getLives() < 150) {
            // Increase the player's current health by 20 points
            state.setLives(state.getLives() + 20);

            // Deduct one heart powerup charge from the player's inventory
            powerups.getHearts().removeCharge();
        }
    }

    void complete() {
        getMyState().setStatus(PlayerStatus.Completed);
        client.sendEvent(new TyperacerStatusEvent(me, getMyState().getStatus()));
    }

    public static enum State {
        Start, Countdown, Playing, Ended
    }

    public static enum CharState {
        Correct, Incorrect
    }

    public long getStartTime() {
        return this.startTime;
    }

    public Stats getStats() {
        return this.stats;
    }

    public int getPoints() {
        return this.points;
    }

    /**
     * Completes all the necessary clean up to have the object safely be destroyed.
     * removes the event listeners and resets the on state change so no rouge ui
     * updates occur
     */
    public void drop() {
        this.client.removeListener(this.keyTypedEvent);
        this.client.removeListener(this.statusEvent);
        this.onStateChange = () -> {
        };
    }
}
