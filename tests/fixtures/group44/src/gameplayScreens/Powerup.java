package gameplayScreens;

import java.awt.Color;
import java.awt.Dimension;

import javax.swing.Box;
import javax.swing.BoxLayout;
import javax.swing.JPanel;

import UIComponents.Subheader;
import UIComponents.Text;
import utils.ColorManager;

/**
 * The Powerup class represents a visual component in the game's Heads Up Display (HUD).
 * It displays information about a specific in-game powerup, including its name, 
 * its associated keyboard shortcut, and how many charges (uses) are currently available.
 * 
 * @author Arielle Tetelbaum
 * @see gameplayScreens.Gameplay
 * @see gameplayScreens.PowerupKey
 */
public class Powerup extends JPanel {
    /** The display name of the powerup. */
    private String name = "";
    /** The integer key code representing the physical keyboard key used to activate it. */
    private int key;
    /** The maximum number of charges this powerup can hold at one time. */
    private int maxUses;
    /** The current number of charges available to the player. */
    private int uses;

    /** The UI text element displaying the ratio of current uses to max uses. */
    private Text text;

    /**
     * Constructs a new Powerup UI component and positions it on the screen.
     *
     * @param x       The X coordinate to place this component.
     * @param y       The Y coordinate to place this component.
     * @param name    The string name of the powerup (e.g., "Boost").
     * @param key     The numerical identifier for the activation key (e.g., 1, 2, or 3).
     * @param uses    The starting amount of charges.
     * @param maxUses The absolute maximum amount of charges allowed.
     * @param fgColor The foreground color used for the text.
     * @param bgColor The background color of the powerup's name plaque.
     */
    public Powerup(int x, int y, String name, int key, int uses, int maxUses, Color fgColor, Color bgColor) {
        // Initialize class state
        this.name = name;
        this.key = key;
        this.maxUses = maxUses;
        this.uses = uses;

        // Make the panel background transparent to blend into the main HUD
        setVisible(true);
        setOpaque(false);
        // Stack elements vertically inside this container
        setLayout(new BoxLayout(this, BoxLayout.Y_AXIS));

        // Create and format the text showing available charges (e.g., "3/3")
        text = new Text(uses + "/" + maxUses, 15);
        text.setAlignmentX(CENTER_ALIGNMENT);
        
        // Create the main title for the powerup
        Subheader pName = new Subheader(name);
        pName.setAlignmentY(CENTER_ALIGNMENT);
        pName.setForeground(fgColor);

        // Construct a distinct background plate for the powerup's name
        JPanel namePanel = new JPanel();
        // Arrange items horizontally within the name plate
        namePanel.setLayout(new BoxLayout(namePanel, BoxLayout.X_AXIS));
        // Force strict dimensions so all powerup boxes are uniformly sized
        namePanel.setPreferredSize(new Dimension(140, 40));
        namePanel.setMaximumSize(new Dimension(140, 40));
        namePanel.setMinimumSize(new Dimension(140, 40));
        namePanel.setForeground(fgColor);
        namePanel.setBackground(bgColor);

        // Generate the visual key indicator graphic
        PowerupKey keyPanel = new PowerupKey(key);

        // Center the name text horizontally inside its background plate using "glue" space
        namePanel.add(Box.createHorizontalGlue());
        namePanel.add(pName);
        namePanel.add(Box.createHorizontalGlue());

        // Assemble the final component vertically: Charge count -> Name Plate -> Key Indicator
        text.setForeground(ColorManager.secondaryBrown);
        add(text);
        add(Box.createVerticalStrut(3)); // Small vertical gap
        setForeground(fgColor);
        add(namePanel);
        add(Box.createVerticalStrut(6)); // Slightly larger vertical gap
        add(keyPanel);

        // Position the completely built component at the specified coordinates
        setBounds(x, y, getPreferredSize().width, getPreferredSize().height);
    }

    /**
     * Dynamically updates the displayed number of charges remaining for this powerup.
     * Called during the main game loop whenever a powerup is gained or consumed.
     *
     * @param newAmount The new total of available uses.
     */
    public void setUses(int newAmount) {
        // Update internal state
        uses = newAmount;

        // Refresh the visual text element
        text.setText(String.format("%d/%d", uses, maxUses));
    }

}