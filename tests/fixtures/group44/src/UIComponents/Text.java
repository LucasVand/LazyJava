package UIComponents;

import java.awt.Color;
import java.awt.Font;

import javax.swing.JLabel;

import utils.ColorManager;
import utils.FontManager;

/**
 * Text object for JLabel text components. Contains constructors for many different
 * customizabilities.
 * 
 * @author Sam Deitz
 */
public class Text extends JLabel {

    /**
     * Create a text element
     * 
     * @param text text
     * @param size size
     */
    public Text(String text, float size) {
        super(text);
        setFont(FontManager.getFont(size));
    }

    /**
     * Create a text element
     * 
     * @param x      x-coordinate
     * @param y      y-coordinate
     * @param color  color of text
     * @param text   text
     * @param font   font
     */
    public Text(int x, int y, Color color, Font font, String text) {
        super(text);
        setFont(font);
        setForeground(color);
        setBounds(x, y, getPreferredSize().width + 40, getPreferredSize().height);
    }

    /**
     * Create a text element
     * 
     * @param x      x-coordinate
     * @param y      y-coordinate
     * @param color  color of text
     * @param size   size of text
     * @param text   text
     */
    public Text(int x, int y, Color color, float size, String text) {
        this(x, y, color, FontManager.getFont(size), text);
    }

    /**
     * Create a text element
     * 
     * @param x      x-coordinate
     * @param y      y-coordinate
     * @param size   size of text
     * @param text   text
     */
    public Text(int x, int y, float size, String text) {
        this(x, y, ColorManager.baseText, FontManager.getFont(size), text);
    }

    /**
     * Create a text element
     * 
     * @param x      x-coordinate
     * @param y      y-coordinate
     * @param text   text
     */
    public Text(int x, int y, String text) {
        this(x, y, ColorManager.baseText, FontManager.getFont(10f), text);
    }

    /**
     * Create a text element
     * 
     * @param x      x-coordinate
     * @param y      y-coordinate
     * @param text   text
     * @param font   font
     */
    public Text(int x, int y, Font font, String text) {
        this(x, y, ColorManager.baseText, font, text);
    }
}
