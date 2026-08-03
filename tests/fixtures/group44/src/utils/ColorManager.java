package utils;

import java.awt.Color;

/**
 * All UI colors used in the game
 * 
 * @author Sam Deitz
 */
public class ColorManager {

    private ColorManager() {}

    /** Default text color (black). */
    public static Color baseText = new Color(0,0,0);

    /** Primary brown color used for main UI elements. */
    public static Color primaryBrown = new Color(132,89,0);

    /** Secondary brown color for accents. */
    public static Color secondaryBrown = new Color(170, 117, 72);

    /** Primary blue color for highlights or buttons. */
    public static Color primaryBlue = new Color(98,135,162);

    /** Sand color used for backgrounds. */
    public static Color primarySand = new Color(233, 223, 199);

    /** Tertiary brown for additional styling. */
    public static Color thirdBrown = new Color(186,141,103);

    /** Boost action color. */
    public static Color boost = new Color(235,178,61);

    /** Text color for boost elements. */
    public static Color boostText = primaryBrown;

    /** Color for adding hearts (health/life). */
    public static Color addHeart = new Color(255,166,196);

    /** Text color for add heart elements. */
    public static Color addHeartText = new Color(157,61,94);

    /** Skip action color. */
    public static Color skip = new Color(127,223,162);

    /** Text color for skip elements. */
    public static Color skipText = new Color(53,125,79);


    /**
     * To avoid issues when importing the color into PlayerStatisticsPage
     * 
     * @param color color object
     * @return hex code
     */
    public static String toHex(Color color) {
        return String.format("#%02x%02x%02x", color.getRed(), color.getGreen(), color.getBlue());
    }
}
