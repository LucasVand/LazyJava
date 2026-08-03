package gameplayScreens;

import java.awt.BasicStroke;
import java.awt.Dimension;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.RenderingHints;
import java.awt.image.BufferedImage;
import java.util.HashMap;
import java.util.Map;

import javax.imageio.ImageIO;
import javax.swing.JPanel;
import javax.swing.Timer;

import UIComponents.Header;
import UIComponents.StyledButton;
import UIComponents.Subheader;
import UIComponents.Text;
import controller.screens.screenEvent.ScreenEventBus;
import playerScreen.PlayerScreenPage;
import saveData.Stats;
import state.clientState.ClientState;
import state.clientState.GameState;
import state.clientState.Player;
import state.clientState.Powerups;
import state.clientState.rankState.PlayerRankingState;
import state.clientState.rankState.RankState;
import state.clientState.rankState.Ranking;
import utils.ColorManager;
import utils.Tuple;

/**
 * The GameOver class represents the post-game screen displayed to players
 * after a typing race has concluded. It displays the race leaderboard,
 * individual player statistics, and provides controls for exiting the lobby
 * or readying up for the next level.
 * 
 * @author Arielle Tetelbaum 
 */
public class GameOver extends JPanel {
    private final int WINDOW_WIDTH = 1200;
    private final int WINDOW_HEIGHT = 700;
    private final int ADDED_POWERUPS = 1;

    private ClientState cState;

    private Tuple<Player, Ranking>[] rankings;

    private StyledButton ready;
    private Text ready_text;
    private Header level_text;
    private int playersReady = 0;

    private Map<String, BufferedImage> boatImages = new HashMap<>();

    /**
     * Constructs a new GameOver panel. Initializes UI components, loads
     * images, sets up leaderboards and statistics, and initializes the
     * timer to track the "ready" status of players in the lobby.
     * @param cs The current state of the client, used to retrieve rankings,
     * statistics, and player data.
     */
    public GameOver(ClientState cs) {
        setPreferredSize(new Dimension(WINDOW_WIDTH, WINDOW_HEIGHT));

        this.cState = cs;
        // Fetch the final race results from the client state
        rankings = cs.getPlayerRankings();

        // Listen for changes in the lobby's rank state (e.g., someone readies up or
        // changes difficulty)
        cs.getRankState().setOnStateChange(() -> {
            calculateReady();
            updateLevelText();
        });

        setBackground(ColorManager.primarySand);
        setFocusable(true);
        setLayout(null); // Absolute positioning

        // Poll the ready state every second to keep the UI up to date
        Timer readyPlayersUpdater = new Timer(1000, e -> calculateReady());
        readyPlayersUpdater.start();

        // Execute visual and data setup routines
        loadImages();
        setupUIComponents();
        setupLeaderboard();
        setupStatistics();
        addPowerups(ADDED_POWERUPS);
    }

    /**
     * Custom painting method to render graphical components on the panel,
     * such as the background and the visual table for the leaderboard.
     * * @param g The graphics context used for drawing.
     */
    @Override
    public void paintComponent(Graphics g) {
        super.paintComponent(g);
        Graphics2D g2 = (Graphics2D) g;

        // Turn on Anti-aliasing for smooth lines
        g2.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);

        // Draw the HUD overlay last
        drawLeaderboard(g2);
    }

    /**
     * Initializes and positions the primary interactive UI components on the
     * screen, such as the Exit button, Restart toggle, and Ready button.
     */
    private void setupUIComponents() {
        add(new Header(35, 60, ColorManager.primaryBrown, "Race Leaderboard"));

        // Buttons
        // Exit button always appears, allowing the player to return to the main menu
        StyledButton exit = new StyledButton("EXIT", StyledButton.ButtonStyle.RECT);
        exit.setBounds(20, 600, exit.getPreferredSize().width, exit.getPreferredSize().height);
        exit.addActionListener(e -> {
            ScreenEventBus.publish("PLAYER_SCREEN", new PlayerScreenPage());
            cState.close();
        });
        add(exit);

        // If the game isn't completely over (i.e., there are more levels to play)
        if (!cState.getRankState().isEnd()) {

            // Display the next difficulty level
            level_text = new Header(35, 480, ColorManager.primaryBlue,
                    "Next Level: Difficulty " + cState.getRankState().getNextDifficulty());
            add(level_text);

            // Button to propose restarting the current level instead of advancing
            StyledButton restart = new StyledButton("Toggle Restart", StyledButton.ButtonStyle.RECT);
            restart.setBounds(100, 600, restart.getPreferredSize().width + 30, restart.getPreferredSize().height);
            restart.addActionListener(e -> cState.getRankState().toggleRestartMode());
            add(restart);

            // Ready button to signal the player is prepared for the next round
            ready = new StyledButton("READY", StyledButton.ButtonStyle.PILL, ColorManager.primaryBlue,
                    ColorManager.primarySand);
            ready.setBounds(470, 460, ready.getPreferredSize().width, ready.getPreferredSize().height);
            ready.addActionListener(e -> {
                handleReady();
            });
            add(ready);

            // Text to show how many players in the lobby are currently ready
            ready_text = new Text(480, 535, ColorManager.primaryBrown, 17,
                    playersReady + "/" + cState.getTotalPlayers() + " Players Ready");
            add(ready_text);
        } else {
            // If the rank state is "end", this was the final level. Show Game Over text
            // instead of next-level controls.
            Header gameOverText = new Header(35, 480, ColorManager.primaryBrown, "Game Over");
            add(gameOverText);
        }

    }

    /**
     * Updates the text displaying the upcoming level difficulty, or
     * indicates if the current level is set to restart based on the
     * rank state.
     */
    private void updateLevelText() {
        // Do nothing if there is no next level
        if (cState.getRankState().isEnd())
            return;

        RankState rs = cState.getRankState();
        // Check if the lobby has decided to restart the current level
        if (rs.isRestartMode()) {
            level_text.setText("Restart Level");
        } else {
            // Otherwise, show the difficulty of the advancing level
            level_text.setText("Next Level: Difficulty " + cState.getRankState().getNextDifficulty());
        }
        revalidate();
        repaint();
    }

    /**
     * Polls the server/client state to calculate how many players have
     * readied up for the next game and updates the UI accordingly.
     */
    private void calculateReady() {
        // Only calculate if we are currently in the ranking phase and the game isn't
        // fully over
        if (cState.getState() != GameState.Ranking || cState.getRankState().isEnd()) {
            return;
        }

        Tuple<Player, PlayerRankingState>[] states = cState.getRankingPlayerState();
        playersReady = 0;

        // Count how many players have their ready flag set to true
        for (int i = 0; i < states.length; i++)
            if (states[i].second.getReady())
                playersReady++;

        // Update the UI text to reflect the current count
        ready_text.setText(playersReady + "/" + cState.getTotalPlayers() + " Players Ready");
        ready_text.revalidate();
        ready_text.repaint();

    }

    /**
     * Awards post-game powerup charges to the local player up to the
     * maximum allowed capacity.
     * * @param x The amount of charges to add for each powerup type.
     */
    private void addPowerups(int x) {
        for (int i = 0; i < x; i++) {
            Powerups pus = cState.getPowerups();
            // Add a charge to each powerup type, capping the maximum at 3
            if (pus.getBoosts().getCharges() < 3)
                pus.getBoosts().addCharges(1);
            if (pus.getHearts().getCharges() < 3)
                pus.getHearts().addCharges(1);
            if (pus.getSkips().getCharges() < 3)
                pus.getSkips().addCharges(1);
        }
    }

    /**
     * Handles the action of the local player clicking the "Ready" button.
     * Toggles the local ready state and updates the button styling to
     * reflect the active state.
     */
    private void handleReady() {
        // Send the toggle command to the state
        cState.getRankState().toggleReady();

        boolean isReady = cState.getMyRankState().second.getReady();
        // Invert the button colors visually to show the toggle state
        if (isReady)
            ready.setNewColors(ColorManager.primarySand, ColorManager.primaryBlue);
        else
            ready.setNewColors(ColorManager.primaryBlue, ColorManager.primarySand);

        ready_text.setText(playersReady + "/" + cState.getTotalPlayers() + " Players Ready");
    }

    /**
     * Initializes and positions the text components for the race leaderboard.
     * Iterates through the player rankings and displays rank, name, peak WPM,
     * finish time, and total points.
     */
    private void setupLeaderboard() {

        // Define column x-coordinates to keep the table aligned
        int[] colX = { 35, 75, 110, 340, 480, 570 };

        // Column Headers
        add(new Text(colX[0], 160, ColorManager.secondaryBrown, 16, "Rank"));
        // add(new Text(colX[1], 160, Color.gray, "***")); MAYBE A 16,DD PALYERS BOAT
        // ICON?
        add(new Text(colX[2], 160, ColorManager.secondaryBrown, 16, "Name"));
        add(new Text(colX[3], 160, ColorManager.secondaryBrown, 16, "Peak WPM"));
        add(new Text(colX[4], 160, ColorManager.secondaryBrown, 16, "Time"));
        add(new Text(colX[5], 160, ColorManager.secondaryBrown, 16, "Score"));

        int startY = 195;
        // Iterate through all ranked players and populate their row data
        for (int i = 0; i < rankings.length; i++) {
            // If the player's time is Long.MAX_VALUE, they didn't finish (DNF)
            String timeString = rankings[i].second.time() == Long.MAX_VALUE ? "DNF"
                    : String.format("%.2fs", rankings[i].second.time() / 1000.);

            // Add text fields for the player's stats at the current row Y coordinate
            add(new Text(colX[0] + 10, startY, ColorManager.primarySand, 20, i + 1 + ""));
            add(new Text(colX[2], startY, ColorManager.primarySand, 20, "" + rankings[i].first.getName()));
            add(new Text(colX[3], startY, ColorManager.primarySand, 20, "" + rankings[i].second.peakWPM() + " wpm"));
            add(new Text(colX[4], startY, ColorManager.primarySand, 20, "" + timeString));
            add(new Text(colX[5], startY, ColorManager.primarySand, 20, "" + rankings[i].second.points()));

            // Move the Y coordinate down for the next player's row
            startY += 50;
        }
    }

    /**
     * Finds the placement (rank) of the local player in the recent match.
     * * @return The 1-based index rank of the local player, or -1 if not found.
     */
    private int getMyPlace() {
        int place = 1;
        // Iterate through rankings until the current client's player ID is found
        for (Tuple<Player, Ranking> ranking : rankings) {
            Player me = cState.getPlayer();
            if (ranking.first.getId().equals(me.getId())) {
                return place;
            }
            place++;
        }
        return -1;
    }

    /**
     * Converts an integer placement into a formatted string with its
     * ordinal suffix (e.g., 1 -> "1st", 2 -> "2nd").
     * * @param place The placement integer to format.
     * 
     * @return The ordinal string representation of the placement.
     */
    private String prefix(int place) {
        // Handle the specific suffixes for top 3 placements
        switch (place) {
            case 1:
                return "1st";
            case 2:
                return "2nd";
            case 3:
                return "3rd";
            default:
                // Default to 'th' for 4th and beyond (Note: does not handle 21st, 22nd, etc.)
                return place + "th";
        }
    }

    /**
     * Initializes and positions the text components displaying the detailed
     * statistics for the local player. Shows placement, WPM metrics,
     * accuracy, errors, points, and earned powerups.
     */
    private void setupStatistics() {
        Stats playerStats = cState.getPlayer().getStats();

        add(new Header(810, 60, ColorManager.primaryBrown, "Your Statistics"));

        // Placement section
        add(new Header(740, 160, ColorManager.primaryBlue, prefix(getMyPlace()) + " Place"));
        add(new Subheader(740, 200, ColorManager.secondaryBrown, "Finish"));

        // Average WPM formatted to 2 decimal places
        String avgWPM = String.format("%.2f wpm", playerStats.getAvgWPM());
        add(new Header(740, 270, ColorManager.primaryBlue, avgWPM));
        add(new Subheader(740, 310, ColorManager.secondaryBrown, "Average WPM"));

        // Accuracy converted to a percentage and formatted to 2 decimal places
        String accuracy = String.format("%.2f%%", playerStats.getAccuracy() * 100.0);
        add(new Header(740, 380, ColorManager.primaryBlue, accuracy));
        add(new Subheader(740, 420, ColorManager.secondaryBrown, "Accuracy"));

        // Current round points
        int points = cState.getPlayerRankings(cState.getPlayer().getId()).second.points();
        add(new Header(1000, 160, ColorManager.primaryBlue, "" + points));
        add(new Subheader(1000, 200, ColorManager.secondaryBrown, "Score"));

        // Highest WPM achieved in the round
        String peakWPM = String.format("%.2f wpm", playerStats.getPeakWPM());
        add(new Header(1000, 270, ColorManager.primaryBlue, peakWPM));
        add(new Subheader(1000, 310, ColorManager.secondaryBrown, "Peak WPM"));

        // Total typing errors
        add(new Header(1000, 380, ColorManager.primaryBlue, "" + playerStats.getErrorCount()));
        add(new Subheader(1000, 420, ColorManager.secondaryBrown, "Errors"));

        // Powerup rewards summary
        add(new Header(740, 480, ColorManager.primaryBrown, "Powerups Gained:"));
        add(new Text(760, 530, ColorManager.boost, 24, "+" + ADDED_POWERUPS + " Boosts"));
        add(new Text(760, 570, ColorManager.addHeart, 24, "+" + ADDED_POWERUPS + " Hearts"));
        add(new Text(760, 610, ColorManager.skip, 24, "+" + ADDED_POWERUPS + " Skips"));

    }

    /**
     * Draws the graphical table for the leaderboard using the provided
     * Graphics2D context. Renders alternating row colors (zebra striping),
     * borders, dividing lines, and the appropriate boat icons.
     * * @param g2 The Graphics2D context to render the leaderboard graphics.
     */
    private void drawLeaderboard(Graphics2D g2) {
        // 1. Setup Table Dimensions
        int startX = 35; // Top-left corner X
        int startY = 180; // Top-left corner Y
        int rowHeight = 50;
        int numRows = 5; // 1 Header row + 5 Player rows
        int totalWidth = 600;

        // 2. Draw Row Backgrounds (Zebra Striping)
        for (int i = 0; i < numRows; i++) {
            int rowY = startY + (i * rowHeight);

            // Alternate row background colors for readability
            if (i % 2 != 0) {
                g2.setColor(ColorManager.secondaryBrown);
                g2.fillRect(startX, rowY, totalWidth, rowHeight);
            } else {
                g2.setColor(ColorManager.thirdBrown);
                g2.fillRect(startX, rowY, totalWidth, rowHeight);
            }
        }

        // 3. Draw the Outer Box and Horizontal Lines
        g2.setColor(ColorManager.primarySand);
        g2.setStroke(new BasicStroke(3.0f)); // Thick outer border
        g2.drawRect(startX, startY, totalWidth, rowHeight * numRows);

        g2.setStroke(new BasicStroke(1.5f)); // Thinner inner horizontal lines
        // Draw dividing lines between rows (skipping the top border)
        for (int i = 1; i < numRows; i++) {
            int lineY = startY + (i * rowHeight);
            g2.drawLine(startX, lineY, startX + totalWidth, lineY);
        }

        // 4. Draw Boat Icons
        for (int i = 0; i < rankings.length; i++) {
            // Fallback to "brown" if the player's color is null
            String color = rankings[i].first.color != null ? rankings[i].first.color.toLowerCase() : "brown";

            BufferedImage boatImg = boatImages.get(color);

            // If the image was loaded successfully, draw it in the row
            if (boatImg != null) {
                int imgY = startY + (i * rowHeight) + 10;
                // Draw the icon in the space reserved at X=70, scaling it to 30x30
                g2.drawImage(boatImg, 70, imgY, 30, 30, null);
            }
        }
    }

    /**
     * Pre-loads the boat image resources into memory for rendering on the
     * leaderboard table based on player color assignments.
     */
    private void loadImages() {
        // List of all possible boat colors
        String[] colors = { "pink", "green", "yellow", "purple", "orange", "black" };

        for (String color : colors) {
            try {
                // Construct the path dynamically based on the color string
                String imagePath = "/resources/images/" + color + "Boat.png";
                BufferedImage img = ImageIO.read(getClass().getResourceAsStream(imagePath));
                // Store in the map using lowercase color as the key for easy retrieval later
                boatImages.put(color.toLowerCase(), img);
            } catch (Exception e) {
                // Fail gracefully if an image is missing, printing an error to the console
                System.err.println("Could not load boat image: " + color + "Boat.png");
            }
        }
    }
}
