package mainMenu;

import java.awt.AlphaComposite;
import java.awt.BorderLayout;
import java.awt.Dimension;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.GridBagLayout;
import java.awt.image.BufferedImage;

import javax.imageio.ImageIO;
import javax.swing.Box;
import javax.swing.JPanel;

import UIComponents.Flexbox;
import UIComponents.PageLayout;
import saveData.HighscoreTable;
import saveData.SaveData;
import utils.ColorManager;

/**
 * The ViewHighScoresPage class is a JPanel that displays the game's high scores.
 * It fetches saved data from the HighscoreTable and renders a visual interface
 * containing a formatted table of player ranks, usernames, and scores over a
 * transparent background image.
 */
public class ViewHighScoresPage extends JPanel {

    /**
     * The background image rendered behind the high score table.
     */
    private BufferedImage bgImage;

    /**
     * Constructs a new ViewHighScoresPage.
     * Initializes the panel's properties, dimensions, and layout. It attempts
     * to load the background image, fetches the current high score data, and
     * delegates the creation of the user interface components.
     */
    public ViewHighScoresPage() {
        setVisible(true);
        setPreferredSize(new Dimension(1200, 700));
        setFocusable(true);
        setLayout(new BorderLayout());
        HighscoreTable tableData = SaveData.getData().getHighscoreTable();

        try {
            // Start with a "/" to look from the root of the JAR, and use getResourceAsStream
            bgImage = ImageIO.read(getClass().getResourceAsStream("/resources/images/islandBg.png"));
        } catch (Exception e) {
            System.err.println("Could not load background image!");
            e.printStackTrace();
        }

        createHighScoreTable(tableData);
        PageLayout.createMenuButton(this);
    }

    /**
     * Constructs the UI layout for the high score table and populates it with data.
     * Creates a static header and dynamically generates rows for each score. 
     * If no scores are present, it displays a fallback message indicating an empty table.
     *
     * @param tableData the HighscoreTable object containing the array of saved high scores
     */
    private void createHighScoreTable(HighscoreTable tableData) {
        JPanel wrapper = new JPanel(new GridBagLayout());
        Flexbox mainPanel = new Flexbox(true);
        mainPanel.setPreferredSize(new Dimension(700, 500));
        mainPanel.setMinimumSize(new Dimension(700, 500));
        mainPanel.setMaximumSize(new Dimension(700, 500));
        wrapper.setOpaque(false);
        mainPanel.setOpaque(false);

        // Scores table container
        Flexbox table = new Flexbox(true);
        table.setAlignmentX(CENTER_ALIGNMENT);
        table.setPreferredSize(new Dimension(700, 500));
        table.setMinimumSize(new Dimension(700, 500));
        table.setMaximumSize(new Dimension(700, 500));
        table.setBackground(ColorManager.primarySand);

        // 1. Static Table Header
        ScoreRecord header = new ScoreRecord("Rank", "Username", "Score", true);
        table.add(header);

        // 2. Dynamic Table Rows
        if (tableData != null && tableData.getHighscores().length > 0) {
            HighscoreTable.Highscore[] scores = tableData.getHighscores();
            
            for (int i = 0; i < scores.length; i++) {
                String rank = String.valueOf(i + 1);
                String username = scores[i].getUsername();
                // Format the double to remove decimals for a cleaner UI display
                String scoreValue = String.format("%.0f", scores[i].getScore());
                
                table.add(new ScoreRecord(rank, username, scoreValue, false));
            }
        } else {
            // Fallback if the table is empty
            table.add(new ScoreRecord("-", "No scores recorded yet", "-", false));
        }

        // Add components to the layout
        mainPanel.add(Box.createVerticalStrut(20));
        mainPanel.add(table);
        wrapper.add(mainPanel);
        add(wrapper, BorderLayout.CENTER);
    }

    /**
     * Overrides the default paintComponent method to render the background image
     * with a specified opacity level, ensuring the UI elements placed on top 
     * remain readable.
     *
     * @param g the Graphics context used for drawing operations
     */
    @Override
    public void paintComponent(Graphics g) {
        super.paintComponent(g);

        if (bgImage != null) {
            Graphics2D g2d = (Graphics2D) g.create();
            float opacity = 0.3f;
            g2d.setComposite(AlphaComposite.getInstance(AlphaComposite.SRC_OVER, opacity));
            g2d.drawImage(bgImage, 0, 0, this.getWidth(), this.getHeight(), null);
            g2d.dispose();
        }
    }
}