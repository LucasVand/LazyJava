package state.clientState;

/**
 * The Powerups class serves as a container grouping together all the distinct 
 * powerup abilities a player can possess in the game. It manages the initialization 
 * and retrieval of the Boost, Heart, and Skip powerup objects.
 * 
 * @author Arielle Tetelbaum
 */
public class Powerups {
    /** The powerup ability that grants the player a temporary speed or score multiplier. */
    Powerup boosts;
    /** The powerup ability that allows the player to instantly complete a word. */
    Powerup skips;
    /** The powerup ability that restores the player's health or lives. */
    Powerup hearts;

    /**
     * Constructs a new collection of powerups for a player, setting up the names, 
     * keyboard hotkeys, and starting inventory for each type.
     *
     * @param initialBoosts The starting number of boost charges.
     * @param initialHearts The starting number of heart (health) charges.
     * @param initialSkips  The starting number of skip charges.
     */
    public Powerups(int initialBoosts, int initialHearts, int initialSkips) {
        // Initialize the Boost powerup, bind it to key '1', and set its starting inventory
        boosts = new Powerup("Boost", 1, initialBoosts);
        
        // Initialize the Heart powerup, bind it to key '2', and set its starting inventory
        hearts = new Powerup("+1 Hearts", 2, initialHearts);
        
        // Initialize the Skip powerup, bind it to key '3', and set its starting inventory
        skips = new Powerup("Skips", 3, initialSkips);
    }

    /**
     * Retrieves the player's Boost powerup object.
     *
     * @return The Powerup instance managing boosts.
     */
    public Powerup getBoosts() {
        return boosts;
    }

    /**
     * Retrieves the player's Skip powerup object.
     *
     * @return The Powerup instance managing skips.
     */
    public Powerup getSkips() {
        return skips;
    }

    /**
     * Retrieves the player's Heart (health) powerup object.
     *
     * @return The Powerup instance managing hearts.
     */
    public Powerup getHearts() {
        return hearts;
    }

}