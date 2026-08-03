package playerScreen;

import java.awt.Dimension;
import java.awt.image.BufferedImage;

import javax.swing.BoxLayout;
import javax.swing.ImageIcon;
import javax.swing.JLabel;
import javax.swing.JPanel;

import UIComponents.Text;
import utils.ColorManager;


/**
 * This class is a UI component for a player icon within the waiting room. It will have the 
 * users boat, name, and show weather or not they are the host
 * 
 * @author Arielle Tetelbaum
 */
public class PlayerIcon extends JPanel {

    /**
     * Boat image to render
     */
    private BufferedImage image = null;

    /**
     * Create a new player icon instance
     * @param playerName player name
     * @param host true if host false otherwise
     * @param image boat image associated to the player
     */
    public PlayerIcon(String playerName, boolean host, BufferedImage image) {
        this.image = image;
        setOpaque(false);
        setLayout(new BoxLayout(this, BoxLayout.Y_AXIS));

        // Host label
        if (host) {
            Text hostTitle = new Text("HOST", 15);
            hostTitle.setAlignmentX(CENTER_ALIGNMENT);
            hostTitle.setForeground(ColorManager.primaryBrown);
            add(hostTitle);
        }

        // player image
        JPanel imagePanel = new JPanel();
        imagePanel.setPreferredSize(new Dimension(70,70));
        imagePanel.setMinimumSize(new Dimension(70,70));
        imagePanel.setMaximumSize(new Dimension(70,70));
        imagePanel.setAlignmentX(CENTER_ALIGNMENT);
        imagePanel.setOpaque(false);
        if (this.image != null) {
            ImageIcon icon = new ImageIcon(this.image);
            JLabel iconWrap = new JLabel(icon);
            iconWrap.setPreferredSize(new Dimension(70,70));
            iconWrap.setMinimumSize(new Dimension(70,70));
            iconWrap.setMaximumSize(new Dimension(70,70));
            imagePanel.add(iconWrap);
        }
        

        // player's name
        Text name = new Text(playerName, 15);
        name.setAlignmentX(CENTER_ALIGNMENT);
        name.setForeground(ColorManager.primaryBrown);

        add(imagePanel);
        add(name);
        revalidate();
    }
}