package UIComponents;

import java.awt.Color;

import utils.ColorManager;
import utils.FontManager;

/**
 * Subheader object for smaller headers. Extends UIComponents.Text
 * 
 * @author Sam Deitz
 * @see UIComponents.Text
 */
public class Subheader extends Text {

    private final static float SUBHEADER_SIZE = 20f;

    /**
     * create subheader text
     * @param text text
     */
    public Subheader(String text) {
        super(text, SUBHEADER_SIZE);
    }
    
    /**
     * create subheader text with absolute positioning
     * @param x x-coordinate
     * @param y y-coordinate
     * @param text text
     */
    public Subheader(int x, int y, String text) {
        super(x, y, ColorManager.baseText, FontManager.getFont(SUBHEADER_SIZE), text);
    }

    /**
     * Create subheader text with absolute positioning and color
     * @param x x coordinate
     * @param y y coordinate
     * @param c color
     * @param text text
     */
    public Subheader(int x, int y, Color c, String text){
        super(x, y, c, FontManager.getFont(SUBHEADER_SIZE), text);
    }
}
