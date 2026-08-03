package gameplayScreens;

import java.awt.BasicStroke;
import java.awt.Dimension;
import java.awt.Graphics;
import java.awt.Graphics2D;

import javax.swing.JPanel;

import UIComponents.Text;
import utils.ColorManager;

/**
 * The PowerupKey class represents a small visual UI component that displays 
 * a keyboard key indicator (e.g., "1", "2", or "3"). It is used to show players 
 * which physical keyboard key they need to press to activate a specific powerup.
 * 
 * @author Arielle Tetelbaum
 * @see gameplayScreens.Powerup
 */
public class PowerupKey extends JPanel {
    
    /**
     * Constructs a new PowerupKey panel displaying the specified number.
     * * @param num The integer number representing the keyboard key mapped to the powerup.
     */
    public PowerupKey (int num) {
        // Make the panel background transparent so the rounded border and underlying UI show through
        setOpaque(false);
        
        // Create the text element for the number with a font size of 15
        Text numText = new Text(""+num, 15);
        numText.setForeground(ColorManager.secondaryBrown);

        // Add the number text to the center of this panel
        add(numText);

        // Constrain the maximum size to force a square aspect ratio based on its preferred height
        setMaximumSize(new Dimension(getPreferredSize().height, getPreferredSize().height));
    }

    /**
     * Custom painting method to draw the rounded, outline border of the key graphic.
     * * @param g The Graphics context used for drawing operations.
     */
    @Override
    public void paintComponent(Graphics g) {
        // Create a copy of the Graphics context to avoid modifying global state
        Graphics2D g2 = (Graphics2D) g.create();

        // Set the thickness of the border line
        g2.setStroke(new BasicStroke(3));
        g2.setColor(ColorManager.secondaryBrown);
        
        // Draw a rounded rectangle that acts as the physical "key" outline
        // We inset by 1 and subtract 2 from width/height to ensure the stroke isn't clipped off the edge
        g2.drawRoundRect(1, 1, getWidth()-2, getHeight()-2, 16, 16);

        // Call the super method to ensure child components (like the Text number) are drawn on top
        super.paintComponent(g);
    }

}