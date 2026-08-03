package gameplayScreens;

import java.awt.Font;
import java.awt.Graphics2D;
import java.awt.image.BufferedImage;

import utils.ColorManager;

/**
 * The Boat class represents a visual player avatar (a boat) within the racing 
 * gameplay screen. It holds the spatial coordinates, graphical image, and 
 * identifying details needed to render the boat on the track.
 * 
 * @author Arielle Tetelbaum
 * @see gameplayScreens.Gameplay
 */
public class Boat {
    /** The current X coordinate of the boat on the screen. */
    public int x;
    /** The current Y coordinate of the boat on the screen. */
    public int y;
    /** The visual graphic representing the boat. */
    public BufferedImage image;
    /** Flag indicating whether this boat belongs to the local player. */
    public boolean isCurrentUser;
    /** The display name of the player controlling this boat. */
    public String name;
    /** The unique identifier for the player. */
    public String playerId;

    /**
     * Constructs a new Boat instance with the specified graphics and identifying information.
     *
     * @param image         The BufferedImage used to render the boat.
     * @param startX        The starting X coordinate.
     * @param startY        The starting Y coordinate.
     * @param isCurrentUser True if this boat is controlled by the local client, false otherwise.
     * @param name          The display name of the player.
     * @param id            The unique player ID string.
     */
    public Boat(BufferedImage image, int startX, int startY, boolean isCurrentUser, String name, String id) {
        // Initialize all instance variables with the provided arguments
        this.image = image;
        this.x = startX;
        this.y = startY;
        this.isCurrentUser = isCurrentUser;
        this.name = name;
        this.playerId = id;
    }

    /**
     * Renders the boat and its associated player name/indicators to the screen.
     *
     * @param g2 The Graphics2D context used for drawing operations.
     */
    public void draw(Graphics2D g2) {
        // Ensure the image has been loaded before attempting to draw it to prevent null pointer errors
        if (image != null)
            // Draw the boat scaled to a 40x40 pixel bounding box at its current (x, y) coordinates
            g2.drawImage(image, x, y, 40, 40, null);

        // g2.setFont(FontManager.getFont(16));
        
        // Visually distinguish between opponent boats and the local player's boat
        if (!isCurrentUser) {
            // For opponents, draw their display name in black text just below their boat
            g2.setColor(ColorManager.primarySand);
            g2.drawString(name, x + 4, y + 50);
        } else {
            // For the local player, draw a green indicator dot above the boat to easily spot it
            g2.setColor(ColorManager.skipText);
            g2.fillOval(x + 15, y - 8, 8, 8);
            // Also display "YOU" instead of their username below the boat
            g2.setFont(new Font("Sans Serif", Font.BOLD, 10));
            g2.drawString("YOU", x + 7, y + 50);
        }
    }

    /**
     * Updates the horizontal position of the boat. Used to move the boat forward 
     * on the track as the player types.
     *
     * @param newX The new X coordinate for the boat.
     */
    public void setX(int newX) {
        // Overwrite the current X position with the updated tracking position
        this.x = newX;
    }
}