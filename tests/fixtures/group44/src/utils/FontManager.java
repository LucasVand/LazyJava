package utils;

import java.awt.Font;
import java.io.InputStream;


/**
 * get IRISH GROVER font for all files
 * 
 * {@snippet :
 *  setFont(FontManager.getFont(16f))
 * }
 * @author Sam Deitz
 */
public class FontManager {

    /**
     * Font
     */
    private static Font font;

    /**
     * Load font after immediately
     */
    static {
        try {
            InputStream is = FontManager.class.getResourceAsStream("/resources/fonts/IrishGrover.ttf");
            if (is != null) {
                // Load the master template (it will be 1pt size)
                font = Font.createFont(Font.TRUETYPE_FONT, is);
            } else {
                System.err.println("Font file not found. Falling back to default.");
                font = new Font("SansSerif", Font.PLAIN, 1);
            }
        } catch (Exception e) {
            e.printStackTrace();
            font = new Font("SansSerif", Font.PLAIN, 1);
        }
    }


    /**
     * Get the IrishGrover font for usage
     * @param size size (in px) -> xf -> 16f
     * @return irish grover font
     */
    public static Font getFont(float size) {
        return font.deriveFont(size);
    }
    
}
