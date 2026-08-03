package parentalControls;

import java.awt.BorderLayout;
import java.awt.Component;
import java.awt.Dimension;
import java.awt.Font;
import java.awt.GridBagLayout;
import java.awt.GridLayout;

import javax.swing.Box;
import javax.swing.JPanel;

import UIComponents.Flexbox;
import UIComponents.Header;
import UIComponents.PageLayout;
import UIComponents.Text;
import controller.screens.screenEvent.ScreenEvent;
import controller.screens.screenEvent.ScreenEventBus;
import saveData.Stats;
import utils.ColorManager;
import utils.FontManager;


/**
 * The PlayerStatisticsPage class represents a GUI panel that displays 
 * detailed typing and game statistics for a specific player.
 * It visualizes data such as Words Per Minute (WPM), accuracy, error count,
 * total play time, and high scores in a clean grid layout.
 */
public class PlayerStatisticsPage extends JPanel {

    /**
     * Constructs the PlayerStatisticsPage for a specific user.
     * Initializes the panel properties, background color, generates the stats UI,
     * and sets up the back navigation button.
     *
     * @param username    The username of the player whose stats are being viewed.
     * @param playerStats The Stats object containing the player's performance data.
     */
    public PlayerStatisticsPage(String username, Stats playerStats) {
        setVisible(true);
        setPreferredSize(new Dimension(1200, 700));
        setFocusable(true);
        setLayout(new BorderLayout());
        setBackground(ColorManager.primarySand);
        setOpaque(true); 

        createStatsUI(playerStats); // Generate and add the main statistics grid UI
        // Add a back button to navigate back to the admin controls screen
        PageLayout.createBackButton(this, e -> {
            ScreenEventBus.publish(ScreenEvent.GO_TO_ADMIN_CONTROLS);
        });
    }

    /**
     * Builds and assembles the graphical user interface for displaying the statistics.
     * Extracts data from the provided Stats object and populates a 4x2 grid of stat blocks.
     *
     * @param stats The Stats object containing the raw data to be displayed.
     */
    private void createStatsUI(Stats stats) {
        // Wrapper used to center the main content vertically and horizontally
        JPanel wrapper = new JPanel(new GridBagLayout());
        wrapper.setOpaque(false);

        // Main vertical flex container for the title and the grid
        Flexbox mainContent = new Flexbox(true);
        mainContent.setOpaque(false);
        mainContent.setAlignmentX(Component.CENTER_ALIGNMENT);

        // Main Title Header
        Header title = new Header("Your Statistics");
        title.setAlignmentX(Component.CENTER_ALIGNMENT);
        title.setFont(getUiFont(64f, 64)); 
        title.setForeground(ColorManager.primaryBrown);

        // Container for the statistics grid
        Flexbox statBox = new Flexbox(true);
        statBox.setPreferredSize(new Dimension(800, 500));
        statBox.setAlignmentX(Component.CENTER_ALIGNMENT);
        statBox.setOpaque(false); 
        statBox.addPadding(40);

        JPanel gridPanel = new JPanel(new GridLayout(4, 2, 40, 30));
        gridPanel.setOpaque(false);

        // Safely extract stats
        double avgWPM = stats != null ? stats.getAvgWPM() : 0.0;
        double peakWPM = stats != null ? stats.getPeakWPM() : 0.0;
        double accuracy = stats != null ? stats.getAccuracy() * 100 : 0.0;
        long errors = stats != null ? stats.getErrorCount() : 0;
        long words = stats != null ? stats.getWordsTyped() : 0;
        
        // Extract playtime in raw milliseconds for our new helper method
        long playTimeMs = stats != null ? stats.getTotalPlayTime() : 0;

        String avgWpmVal = String.format("%.2f WPM", avgWPM);
        String peakWpmVal = String.format("%.2f WPM", peakWPM);
        String accVal = String.format("%.2f%%", accuracy);
        String errVal = String.valueOf(errors);
        
        // Pass the raw milliseconds to our dynamic formatter
        String timeVal = formatPlayTime(playTimeMs);
        
        String pointsVal = "" + stats.getHighscore();
        String levelVal = "" + stats.getHighestDifficulty(); 
        String wordsVal = String.valueOf(words);

        // Row 1
        gridPanel.add(createStatBlock(avgWpmVal, "Average WPM"));
        gridPanel.add(createStatBlock(peakWpmVal, "Peak WPM"));
        
        // Row 2
        gridPanel.add(createStatBlock(accVal, "Accuracy"));
        gridPanel.add(createStatBlock(errVal, "Errors"));
        
        // Row 3
        gridPanel.add(createStatBlock(timeVal, "Total Time Played"));
        gridPanel.add(createStatBlock(pointsVal, "High Score")); 
        
        // Row 4
        gridPanel.add(createStatBlock(levelVal, "Highest Level Reached")); 
        gridPanel.add(createStatBlock(wordsVal, "Words Typed"));

        statBox.add(gridPanel);
        statBox.add(Box.createVerticalGlue());

        mainContent.add(title);
        mainContent.add(Box.createVerticalStrut(40)); 
        mainContent.add(statBox);

        wrapper.add(mainContent);
        add(wrapper, BorderLayout.CENTER);
    }

    /**
     * Helper method to generate a visually consistent block representing a single statistic.
     * Displays a large value on top and a smaller descriptor label underneath.
     *
     * @param topText    The numerical value or main string to display (e.g., "45.2 WPM").
     * @param bottomText The descriptive label for the statistic (e.g., "Average WPM").
     * @return A styled JPanel (Flexbox) containing the formatted text components.
     */
    private JPanel createStatBlock(String topText, String bottomText) {
        Flexbox block = new Flexbox(true);
        block.setOpaque(false);
        
        // The main numerical value
        Header mainText = new Header(topText);
        mainText.setAlignmentX(Component.LEFT_ALIGNMENT);
        mainText.setFont(getUiFont(42f, 42)); // Blue values
        mainText.setForeground(ColorManager.primaryBlue);
        
        // The descriptive subtext label
        Text subText = new Text(bottomText, 18);
        subText.setAlignmentX(Component.LEFT_ALIGNMENT);
        subText.setFont(getUiFont(24f, 24)); // Brown labels
        subText.setForeground(ColorManager.primaryBrown);
        
        // Stack the text components vertically
        block.add(mainText);
        block.add(Box.createVerticalStrut(2)); 
        block.add(subText);
        
        return block;
    }

    /**
     * Dynamically formats a duration in milliseconds into a readable string 
     * representing seconds, minutes, or hours depending on the total length.
     *
     * @param timeInMillis The total play time in milliseconds.
     * @return A formatted time string with its corresponding unit.
     */
    private String formatPlayTime(long timeInMillis) {
        if (timeInMillis == 0) {
            return "0.00 seconds";
        }
        
        // Convert to seconds first
        double totalSeconds = timeInMillis / 1000.0;
        
        // If it's less than a minute, display in seconds
        if (totalSeconds < 60.0) {
            return String.format("%.2f seconds", totalSeconds);
        } 
        
        // If it's more than a minute, convert to minutes
        double totalMinutes = totalSeconds / 60.0;
        
        // If it's less than an hour, display in minutes
        if (totalMinutes < 60.0) {
            return String.format("%.2f minutes", totalMinutes);
        } else {
            // Otherwise, convert to hours
            double totalHours = totalMinutes / 60.0;
            return String.format("%.2f hours", totalHours);
        }
    }
    
    /**
     * Helper method to retrieve a custom UI font, providing a standard 
     * Serif fallback if the desired custom font cannot be loaded.
     *
     * @param preferredSize The desired size of the custom font as a float.
     * @param fallbackSize  The desired size of the fallback font as an integer.
     * @return The requested Font object, or a default Serif font if unavailable.
     */
    private Font getUiFont(float preferredSize, int fallbackSize) {
        Font customFont = FontManager.getFont(preferredSize);
        if (customFont.getFamily().equalsIgnoreCase("SansSerif")) {
            return new Font("Serif", Font.PLAIN, fallbackSize);
        }
        return customFont;
    }
}