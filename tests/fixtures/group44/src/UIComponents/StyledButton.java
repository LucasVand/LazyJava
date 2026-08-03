package UIComponents;

import java.awt.BasicStroke;
import java.awt.Color;
import java.awt.Font;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.RenderingHints;

import javax.swing.BorderFactory;
import javax.swing.JButton;

import utils.ColorManager;
import utils.FontManager;

/**
 * This class provides a button component. It can be styled in three
 * different ways: Rectangle, Rounded Rectangle, or Horizontal Ellipse/Pill
 * 
 * @author Sam Deitz
 */
public class StyledButton extends JButton {

    /**
     * Determines style of button
     */
    public enum ButtonStyle {
        RECT,
        ROUNDED_RECT,
        PILL
    }

    /**
     * The style of this button
     */
    private final ButtonStyle style;

    /**
     * The font of the text in this button
     */
    private final Font customFont;

    /**
     * Background color for this button
     */
    private Color bgColor;

    /**
     * Text color for this button
     */
    private Color fgColor;


    /**
     * Create a styled button component.
     * 
     * @param text button content
     * @param style enum for button style
     */
    public StyledButton(String text, ButtonStyle style) {
        this(text, style, ColorManager.primarySand, ColorManager.primaryBrown);
    }

    /**
     * Create a styled button component that specifys font and background color
     * 
     * {@snippet :
     * RoundedRectButton r = new RoundedRectButton("Hello", StyledButton.ButtonStyle.RECT);
     * panel.add(r);
     * }
     * 
     * @param text button content
     * @param style enum for button style
     * @param fontColor color of the button content
     * @param backgroundColor color of the button background
     */
    public StyledButton(String text, ButtonStyle style, Color fontColor, Color backgroundColor) {
        super(text);
        this.style = style;
    
        // apply different styles if the button is a PILL
        if (style == ButtonStyle.PILL) {
            // bigger text
            customFont = FontManager.getFont(36f);

            // more padding
            setBorder(BorderFactory.createEmptyBorder(15,30,15, 30));
        } else {
            customFont = FontManager.getFont(24f);
            setBorder(BorderFactory.createEmptyBorder(10,16,10,16));
        }

        // save colors
        fgColor = fontColor;
        bgColor = backgroundColor;

        // Remove default button look
        setContentAreaFilled(false);
        setBorderPainted(false);
        setFocusPainted(false);
        setOpaque(false);


        // Font
        setFont(customFont);
        setForeground(fontColor);

        // UI events for hover and press
        this.getModel().addChangeListener(e -> {
            if (getModel().isPressed()) {
                setForeground(ColorManager.primarySand); 
            } else if (getModel().isRollover()) {
                setForeground(ColorManager.primarySand); 
            } else {
                // Revert to normal color when the mouse leaves
                setForeground(this.fgColor); 
            }
        });
    }

    /**
     * Change the colors of this button
     * @param fgColor content color
     * @param bgColor background color
     */
    public void setNewColors(Color fgColor, Color bgColor) {
        this.bgColor = bgColor;
        this.fgColor = fgColor; 
        this.setForeground(fgColor);
        this.repaint(); 
    }

    @Override
    public void paintComponent(Graphics g) {
        Graphics2D g2 = (Graphics2D) g.create();
        g2.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);

        Color currBgColor = bgColor;
        Color currFgColor = fgColor;

        // hovering the button
        if (getModel().isRollover()) {
            currBgColor = ColorManager.primaryBlue;
            currFgColor = ColorManager.primarySand;
        }


        // ROUNDED RECTANGLE BUTTON: MAIN MENU
        if (style == ButtonStyle.ROUNDED_RECT) {
            // button background
            g2.setColor(currBgColor);
            g2.fillRoundRect(0, 0, getWidth(), getHeight(), 16, 16);

            g2.setColor(currFgColor);
            g2.setStroke(new BasicStroke(3));
            g2.drawRoundRect(1, 1, getWidth()-2, getHeight()-2, 16, 16);

            super.paintComponent(g);
        }

        // RECTANGLE BUTTON: BACK
        if (style == ButtonStyle.RECT) {
            // button background
            g2.setColor(currBgColor);
            g2.fillRect(0, 0, getWidth(), getHeight());

            g2.setColor(currFgColor);
            g2.setStroke(new BasicStroke(3));
            g2.drawRect(1, 1, getWidth()-2, getHeight()-2);

            super.paintComponent(g);
        }

        // PILL BUTTON (OVAL): READY
        if (style == ButtonStyle.PILL) {
            // button background
            g2.setColor(currBgColor);
            g2.fillOval(0, 0, getWidth(), getHeight());

            g2.setColor(currFgColor);
            g2.setStroke(new BasicStroke(3));
            g2.drawOval(1, 1, getWidth()-2, getHeight()-2);

            super.paintComponent(g);
        }
        
    }
}
