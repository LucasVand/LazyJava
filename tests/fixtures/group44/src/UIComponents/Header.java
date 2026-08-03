package UIComponents;

import java.awt.Color;

import utils.ColorManager;
import utils.FontManager;

/**
 * Header object for large header text extends UIComponents.Text
 * 
 * @author Sam Deitz
 * @see UIComponents.Text
 */
public class Header extends Text {

    private final static float HEADER_SIZE = 35f;
    
    /**
     * create Header text
     * @param text text
     */
    public Header(String text) {
        super(text, HEADER_SIZE);
    }

    /**
     * Create header text with coordinates
     * @param x x coordinate
     * @param y y coordinate
     * @param c Color of text
     * @param text the text
     */
    public Header(int x, int y, Color c, String text) {
        super(x, y, c,  FontManager.getFont(HEADER_SIZE), text);
    }

    /**
     * create header text with absolute positioning
     * @param x x-coordinate
     * @param y y-coordinate
     * @param text text
     */
    public Header(int x, int y, String text) {
        super(x, y, ColorManager.baseText, FontManager.getFont(HEADER_SIZE), text);
    }

}
