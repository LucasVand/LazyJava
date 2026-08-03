package state.clientState;

/**
 * The Powerup class serves as a data model representing an in-game ability or item.
 * It tracks the powerup's conceptual name, the keyboard key used to trigger it, 
 * and the number of times the player can currently use it.
 * 
 * @author Arielle Tetelbaum
 */
public class Powerup {
    /** The display name of the powerup (e.g., "Boost" or "Heart"). */
    String name;
    /** The numerical identifier representing the keyboard key mapped to this powerup. */
    int key;
    /** The current number of times this powerup can be used by the player. */
    int charges;

    /**
     * Constructs a new Powerup data object.
     *
     * @param name           The name of the powerup.
     * @param key            The integer key mapping for activation.
     * @param initialCharges The starting number of uses the player has.
     */
    public Powerup(String name, int key, int initialCharges) {
        // Initialize the class fields with the provided arguments
        this.name = name;
        this.key = key;
        this.charges = initialCharges;
    }

    /**
     * Retrieves the current number of charges available for this powerup.
     *
     * @return The number of remaining charges.
     */
    public int getCharges() {
        return charges;
    }

    /**
     * Directly sets the available charges to a specific number.
     *
     * @param num The exact number of charges to assign.
     */
    public void setCharges(int num) {
        // Overwrite the current charge count
        charges = num;
    }

    /**
     * Increases the current number of charges by the specified amount.
     * Used when a player earns a reward or picks up an item.
     *
     * @param num The number of charges to add.
     */
    public void addCharges(int num) {
        // Add the new amount to the existing total
        charges += num;
    }

    /**
     * Decreases the available charges by exactly one.
     * Typically called when the player successfully activates the powerup.
     */
    public void removeCharge() {
        // Subtract one from the current charge count
        charges--;
    }

    /**
     * Retrieves the keyboard key identifier mapped to this powerup.
     *
     * @return The integer key code for activation.
     */
    public int getKey() {
        return key;
    }
}