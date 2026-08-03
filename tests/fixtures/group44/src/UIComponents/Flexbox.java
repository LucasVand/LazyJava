package UIComponents;

import javax.swing.BorderFactory;
import javax.swing.BoxLayout;
import javax.swing.JPanel;

/**
 * JPanel with a BoxLayout for creating responsive pages.
 * 
 * @author Sam Deitz
 */
public class Flexbox extends JPanel {
    
    /**
     * Initialize a flexbox object. Creates a JPanel with a BoxLayout over the X_AXIS
     */
    public Flexbox() {
        setLayout(new BoxLayout(this, BoxLayout.X_AXIS));
        revalidate();
    }

    /**
     * Initialize a flexbox object. Creates a JPanel with a BoxLayout over the y axis or x axis
     * @param vertical true if y axis false otherwise
     */
    public Flexbox(boolean vertical) {
        setLayout(new BoxLayout(this, vertical ? BoxLayout.Y_AXIS : BoxLayout.X_AXIS));
    }

    /**
     * Add inner padding to the flexbox
     * @param padding amount of padding
     */
    public void addPadding(int padding) {
        setBorder(BorderFactory.createEmptyBorder(padding, padding, padding, padding));
    }
}
