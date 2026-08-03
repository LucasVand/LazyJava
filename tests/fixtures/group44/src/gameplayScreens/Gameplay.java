package gameplayScreens;

import java.awt.BasicStroke;
import java.awt.Color;
import java.awt.Dimension;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.RenderingHints;
import java.awt.event.KeyEvent;
import java.awt.event.KeyListener;
import java.awt.image.BufferedImage;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import javax.imageio.ImageIO;
import javax.swing.JPanel;
import javax.swing.JTextPane;
import javax.swing.Timer;
import javax.swing.text.Style;
import javax.swing.text.StyleConstants;
import javax.swing.text.StyleContext;
import javax.swing.text.StyledDocument;

import UIComponents.Header;
import UIComponents.StyledButton;
import UIComponents.Subheader;
import UIComponents.Text;
import controller.screens.screenEvent.ScreenEventBus;
import playerScreen.PlayerScreenPage;
import state.clientState.ClientState;
import state.clientState.GameState;
import state.clientState.Player;
import state.clientState.Powerups;
import state.clientState.typeracerState.PlayerTyperacerState;
import state.clientState.typeracerState.TyperacerState;
import utils.ColorManager;
import utils.FontManager;
import utils.Tuple;

/**
 * The Gameplay class is the primary visual component for the active typing race.
 * It renders the typing track, player boats, real-time typing progress, health bars, 
 * and powerups. It also captures keyboard input to drive the game state forward.
 * 
 * @author Arielle Tetelbaum
 * @see gameplayScreens.Boat
 * @see gameplayScreens.Powerup
 */
public class Gameplay extends JPanel {

    // =========================================
    // 1. CONSTANTS (Colors, Sizes, Positions)
    // =========================================
    private final int WINDOW_WIDTH = 1200;
    private final int WINDOW_HEIGHT = 700;

    private final Color COLOR_WATER = new Color(137, 207, 240);
    private final Color COLOR_WAVES = new Color(115, 147, 179);
    private final Color COLOR_TRACK_BORDER = new Color(210, 43, 43);

    private final int BOAT_START_X = 40;
    private final int BOAT_END_X = 450;

    // =========================================
    // 2. GAME STATE & ASSETS
    // =========================================
    private ClientState cState;

    // UI Label References
    private Subheader scoreLabel;
    private Text speedLabel;
    private Subheader timeLabel;

    private Powerup boostPowerup;
    private Powerup skipPowerup;
    private Powerup livesPowerup;

    private List<Boat> boats;
    private Map<String, BufferedImage> boatImages = new HashMap<>();
    private BufferedImage disconnected;

    private Tuple<Player, PlayerTyperacerState>[] states;

    private JTextPane textDisplay;

    // =========================================
    // 4. CONSTRUCTOR
    // =========================================
    /**
     * Constructs the main Gameplay screen panel. Initializes the connection to the client
     * state, sets up the UI components, loads visual assets, and begins the game loop timer.
     * @param cs The current global client state containing player and game data.
     */
    public Gameplay(ClientState cs) {
        cState = cs;

        setPreferredSize(new Dimension(WINDOW_WIDTH, WINDOW_HEIGHT));
        setBackground(ColorManager.primarySand);
        setFocusable(true);
        setLayout(null); // Absolute positioning

        boats = new ArrayList<>();

        // Safely fetch the typeracer states for all players in the lobby
        if (cs != null && cs.getTyperacerState() != null) {
            states = cs.getTyperacerPlayerState();
        }

        // Initialize everything via helper methods
        loadImages();
        setupBoats();
        setupUIComponents();
        setupKeyListener();

        // Render the initial state of the typing paragraph
        updateWordDisplay();

        // Start a timer to poll the game state and update the UI (roughly every second)
        Timer t = new Timer(900, (e) -> {
            update();
        });
        t.start();

        // Force a redraw whenever the underlying typeracer state notifies of a change
        cs.getTyperacerState().setOnStateChange(() -> {
            this.repaint();
        });
    }

    // =========================================
    // 5. SETUP METHODS
    // =========================================
    /**
     * Initializes and positions all the static and dynamic text labels, buttons,
     * powerup icons, and the central typing text area (JTextPane).
     */
    private void setupUIComponents() {
        // Labels
        add(new Header(35, 30, ColorManager.primaryBrown,
                String.format("Type Racer: Difficulty %d", cState.getTyperacerState().getDifficulty())));
        add(new Subheader(35, 115, ColorManager.secondaryBrown, "Go!"));
        add(new Subheader(440, 115, ColorManager.secondaryBrown, "Finish"));

        timeLabel = new Subheader(35, 515, ColorManager.primaryBrown, "TIME LEFT: **s");
        add(timeLabel);

        scoreLabel = new Subheader(380, 515, ColorManager.primaryBrown, "SCORE: 0");
        add(scoreLabel);

        speedLabel = new Text(630, 70, ColorManager.primaryBrown, 30, "SPEED: *** wpm");
        add(speedLabel);

        // Fetch user's currently available powerups and construct their visual widgets
        Powerups powerups = cState.getPowerups();
        boostPowerup = new Powerup(630, 530, "Boost", 1, powerups.getBoosts().getCharges(), 3, ColorManager.boostText,
                ColorManager.boost);
        livesPowerup = new Powerup(790, 530, "+20 Health", 2, powerups.getHearts().getCharges(), 3,
                ColorManager.addHeartText, ColorManager.addHeart);
        skipPowerup = new Powerup(950, 530, "Skip Word", 3, powerups.getSkips().getCharges(), 3, ColorManager.skipText,
                ColorManager.skip);

        add(boostPowerup);
        add(livesPowerup);
        add(skipPowerup);

        // Buttons
        StyledButton exit = new StyledButton("EXIT", StyledButton.ButtonStyle.RECT);
        exit.setBounds(20, 600, exit.getPreferredSize().width, exit.getPreferredSize().height);
        exit.addActionListener(e -> {
            ScreenEventBus.publish("PLAYER_SCREEN", new PlayerScreenPage());
            cState.close();
        });
        add(exit);

        // PARAGRAPH/TEXT DISPLAY SETUP:
        // Use a JTextPane to allow rich text styling (colors for correct/incorrect letters)
        textDisplay = new JTextPane();
        textDisplay.setEditable(false);
        textDisplay.setFocusable(false); // keep focus on main JPanel to catch key presses
        textDisplay.setOpaque(false);
        textDisplay.setFont(FontManager.getFont(19));

        textDisplay.setBounds(630, 170, 470, 400);

        add(textDisplay);

    }

    /**
     * Attaches a key listener to the main panel to capture typing input from the user.
     * Bypasses control characters and forwards valid keystrokes to the game state.
     */
    private void setupKeyListener() {

        this.addKeyListener(new KeyListener() {
            @Override
            public void keyTyped(KeyEvent e) {
                char ch = e.getKeyChar();
                // Ignore shift, ctrl, backspace, etc.
                if (Character.isISOControl(ch))
                    return;

                if (cState != null && cState.getTyperacerState() != null) {
                    // Prevent typing if the player's boat has died
                    if (cState.getTyperacerState().getMyState().getStatus() == PlayerTyperacerState.PlayerStatus.Dead) {
                        return;
                    }
                    // Pass the typed character to the game logic
                    cState.getTyperacerState().keyTyped(ch);
                }

                // Immediately update HUD to reflect the stroke
                update();

            }

            @Override
            public void keyPressed(KeyEvent e) {
            }

            @Override
            public void keyReleased(KeyEvent e) {
            }
        });
        // Ensure the panel can actually receive the key events
        setFocusable(true);

        // Legacy/Commented out code for text pane specific listening
        // textDisplay.addKeyListener(new KeyAdapter() {
        // @Override
        // public void keyTyped(KeyEvent e) {
        // char ch = e.getKeyChar();
        // if (Character.isISOControl(ch))
        // return;
        //
        // if (cState != null && cState.getTyperacerState() != null) {
        // System.out.println("In here");
        // cState.getTyperacerState().keyTyped(ch);
        // }
        //
        // update();
        // }
        // });
    }

    /**
     * Triggered by the timer and key events. Fetches the latest statistics 
     * (score, time, WPM, powerup charges) from the state and updates the UI labels.
     */
    private void update() {
        if (cState != null && cState.getState() == GameState.Typeracer) {
            TyperacerState ts = cState.getTyperacerState();

            scoreLabel.setText("SCORE: " + ts.getPoints());

            // Convert milliseconds to seconds for display
            timeLabel.setText("TIME LEFT: " + ts.timeRemaining() / 1000 + " s");

            String wpm = String.format("%d wpm", (int) ts.getStats().getAvgWPM());
            speedLabel.setText("SPEED: " + wpm);

            // Sync visual powerup charges with actual state
            boostPowerup.setUses(cState.getPowerups().getBoosts().getCharges());
            livesPowerup.setUses(cState.getPowerups().getHearts().getCharges());
            skipPowerup.setUses(cState.getPowerups().getSkips().getCharges());

            updateWordDisplay();
            repaint();
        }
    }

    /**
     * Rebuilds the contents of the JTextPane document character by character.
     * Applies specific colors to characters based on whether they were typed 
     * correctly, incorrectly, are next to be typed, or remain untyped.
     */
    private void updateWordDisplay() {
        if (cState == null || cState.getTyperacerState() == null)
            return;

        TyperacerState ts = cState.getTyperacerState();
        String paragraph = ts.getParagraph();
        int currPos = ts.getMyState().getPosition();
        ArrayList<TyperacerState.CharState> history = ts.getCharState();

        StyledDocument doc = textDisplay.getStyledDocument();

        try {
            // Clear the existing text
            doc.remove(0, doc.getLength());

            // Create base style
            Style def = StyleContext.getDefaultStyleContext().getStyle(StyleContext.DEFAULT_STYLE);

            // Setup custom styles for text feedback
            Style correct = doc.addStyle("correct", def);
            StyleConstants.setForeground(correct, new Color(46, 204, 113));

            Style incorrect = doc.addStyle("incorrect", def);
            StyleConstants.setForeground(incorrect, Color.WHITE);
            StyleConstants.setBackground(incorrect, new Color(231, 76, 60));

            Style current = doc.addStyle("current", def);
            StyleConstants.setForeground(current, ColorManager.primaryBrown);
            StyleConstants.setUnderline(current, true);

            Style untyped = doc.addStyle("untyped", def);
            StyleConstants.setForeground(untyped, ColorManager.secondaryBrown);

            // Iterate over every character in the target paragraph
            for (int i = 0; i < paragraph.length(); i++) {
                String ch = String.valueOf(paragraph.charAt(i));

                if (i < currPos) { // alr typed
                    // Check history to see if they got this character right or wrong
                    if (history.get(i) == TyperacerState.CharState.Correct) {
                        doc.insertString(doc.getLength(), ch, correct);
                    } else {
                        doc.insertString(doc.getLength(), ch, incorrect);
                    }
                } else if (i == currPos) { // curr letter needed to be typed
                    doc.insertString(doc.getLength(), ch, current);
                } else {
                    // not typed yet
                    doc.insertString(doc.getLength(), ch, untyped);
                }
            }

        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    /**
     * Loads the visual assets (boat images and disconnect icons) from the 
     * resource folder into memory to be used during the paint cycle.
     */
    private void loadImages() {
        String[] colors = { "pink", "green", "yellow", "purple", "orange", "black" };

        // Load all possible boat colors into a map
        for (String color : colors) {
            try {
                String imagePath = "/resources/images/" + color + "Boat.png";
                boatImages.put(color.toLowerCase(), ImageIO.read(getClass().getResourceAsStream(imagePath)));
            } catch (Exception e) {
                System.err.println("Could not load boat image: " + color + "Boat.png");
                e.printStackTrace();
            }
        }

        // Load the special icon used when a player disconnects
        try {
            disconnected = ImageIO.read(getClass().getResourceAsStream("/resources/images/disconnected.png"));
        } catch (Exception e) {
            System.err.println("Could not load disconnected icon");
            e.printStackTrace();
        }
    }

    // =========================================
    // 6. RENDERING (Painting)
    // =========================================
    /**
     * Standard Swing painting method. Called whenever the panel needs to be redrawn.
     * Manages the strict render ordering of background, track, boats, and HUD overlays.
     * * @param g The Graphics context for drawing.
     */
    @Override
    public void paintComponent(Graphics g) {
        // Prevent drawing if we aren't actively in the typeracer state
        if (cState.getState() != GameState.Typeracer) {
            return;
        }
        requestFocus();

        super.paintComponent(g);
        Graphics2D g2 = (Graphics2D) g;

        // Turn on Anti-aliasing for smooth lines
        g2.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);

        // Draw the background layers in order
        drawWater(g2);
        drawTrackBorders(g2);

        // Draw the entities
        drawBoats(g2);

        // Draw the HUD overlay last
        drawHealthBar(g2);

    }

    /**
     * Overrides the top-level paint method to draw an overarching "YOU DIED" 
     * screen tint that sits on top of all child components and text displays.
     * * @param g The Graphics context for drawing.
     */
    @Override
    public void paint(Graphics g) {
        super.paint(g);

        // If the current player has died, draw a transparent overlay over the right half of the screen
        if (cState != null && cState.getState() == GameState.Typeracer) {
            PlayerTyperacerState myState = cState.getTyperacerState().getMyState();

            if (myState != null && myState.getStatus() == PlayerTyperacerState.PlayerStatus.Dead) {
                Graphics2D g2 = (Graphics2D) g;
                g2.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);

                // Semi-transparent screen tint
                g2.setColor(new Color(233, 223, 199, 170)); 
                g2.fillRect(600, 0, WINDOW_WIDTH - 600, WINDOW_HEIGHT);

                // Red "YOU DIED" banner box
                g2.setColor(new Color(210, 43, 43));
                g2.fillRect(820, 270, 120,50);

                // Text
                g2.setColor(Color.white);
                g2.setFont(FontManager.getFont(20));
                g2.drawString("YOU DIED", 838, 303);
            }
        }
    }

    // --- Drawing Helpers ---
    
    /**
     * Calculates the position of each boat based on typing progress and draws them.
     * Also handles drawing overlays if a player is dead or disconnected.
     * * @param g2 The Graphics2D context.
     */
    private void drawBoats(Graphics2D g2) {
        if (cState.getState() != GameState.Typeracer) {
            return;
        }
        // Total physical pixel distance the boat can travel
        int boatLineLength = BOAT_END_X - BOAT_START_X;
        double paragraphLength = (double) cState.getTyperacerState().getParagraph().length();
        int spacingY = 70;
        int i = 0;

        for (Boat boat : boats) {
            PlayerTyperacerState state = cState.getTyperacerState().getPlayerState(boat.playerId);
            
            // Calculate percentage of paragraph completed
            double percent = (double) state.getPosition() / paragraphLength;
            
            // Translate percentage into physical X coordinates
            int currX = BOAT_START_X + (int) (percent * (double) boatLineLength);
            int currY = 160 + (i * spacingY);
            
            // Update boat model and draw it
            boat.setX(currX);
            boat.draw(g2);

            // Draw status overlays on top of the boat if necessary
            if (!states[i].first.isConnected()) {
                g2.drawImage(disconnected, currX, currY, 40, 40, null);  
            }
            else if (state.getStatus() == PlayerTyperacerState.PlayerStatus.Dead) {
                drawRedX(g2, currX, currY);
            }

            i++;
        }
    }

    /**
     * Helper to draw a red 'X' over a boat, indicating the player has died.
     * * @param g2 The Graphics2D context.
     * @param x  The starting X coordinate.
     * @param y  The starting Y coordinate.
     */
    private void drawRedX(Graphics2D g2, int x, int y) {
        g2.setColor(Color.RED);
        g2.setStroke(new BasicStroke(5));

        // Draw crossing lines
        g2.drawLine(x, y, x+40, y+40);
        g2.drawLine(x, y+40, x+40, y);
    }

    /**
     * Draws the blue pool background and animated wave lines behind the boats.
     * * @param g2 The Graphics2D context.
     */
    private void drawWater(Graphics2D g2) {
        // Pool Background
        g2.setColor(COLOR_WATER);
        g2.fillRect(35, 150, 465, 350);

        // Wavy Lines using Sine waves
        g2.setStroke(new BasicStroke(3.0f));
        g2.setColor(COLOR_WAVES);

        int xPoints = 500;
        int yOffset = 220;
        int amplitude = 6;
        double frequency = 0.08;

        // Draw 4 distinct wave rows
        for (int i = 0; i < 4; i++) {
            for (int x = 35; x < xPoints; x++) {
                int y1 = (int) (Math.sin(x * frequency) * amplitude) + (yOffset + 70 * i);
                int y2 = (int) (Math.sin((x + 1) * frequency) * amplitude) + (yOffset + 70 * i);
                g2.drawLine(x, y1, x + 1, y2);
            }
        }
    }

    /**
     * Draws the borders enclosing the racing track and the vertical finish line.
     * * @param g2 The Graphics2D context.
     */
    private void drawTrackBorders(Graphics2D g2) {
        g2.setColor(COLOR_TRACK_BORDER);
        g2.setStroke(new BasicStroke(3));

        // Top and Bottom borders
        g2.drawLine(35, 150, 500, 150);
        g2.drawLine(35, 500, 500, 500);

        // Finish Line
        g2.drawLine(440, 150, 440, 500);
    }

    /**
     * Renders the local player's health bar UI in the top right, altering 
     * color based on how many lives are remaining.
     * * @param g2 The Graphics2D context.
     */
    private void drawHealthBar(Graphics2D g2) {
        int maxHealth = 150;
        int currHealth = cState.getTyperacerState().getMyState().getLives();

        int barHeight = 25;
        int startX = 950;
        int y = 70;

        // Draw gray background indicating lost health
        g2.setColor(ColorManager.secondaryBrown);
        g2.fillRect(startX,y,maxHealth,barHeight);

        // Calculate remaining health fill
        int healthRemaining = Math.max(0, currHealth);
        int damageTaken = maxHealth - healthRemaining;

        int currX = startX + damageTaken;

        // Change health bar color based on remaining health thresholds
        if (currHealth > 75) {
            g2.setColor(ColorManager.skip); // green
        } else if (currHealth > 37) {
            g2.setColor(Color.orange);
        } else {
            g2.setColor(new Color(210, 43, 43)); // Red
        }

        // Draw active health segment
        g2.fillRect(currX, y, healthRemaining, barHeight);

        // Draw outline border
        g2.setColor(ColorManager.primaryBrown);
        g2.setStroke(new BasicStroke(3));
        g2.drawRect(startX,y,maxHealth,barHeight);
        
    }

    /**
     * Maps the underlying state data for each player to visual Boat instances
     * to prepare them for drawing. Applies the correct color image per player.
     */
    private void setupBoats() {
        int spacingY = 70;
        int i = 0;
        
        for (Tuple<Player, PlayerTyperacerState> s : states) {
            boolean isUser = cState.isMe(s.first.id);
            
            String playerColor = s.first.color != null ? s.first.color.toLowerCase() : "pink";
            
            // Safely fetch the color image, falling back to pink if missing
            BufferedImage correctBoatImg = boatImages.getOrDefault(playerColor, boatImages.get("pink"));
            
            // Add the new visual boat object tracking its assigned player ID
            boats.add(new Boat(correctBoatImg, BOAT_START_X, 160 + (i * spacingY), isUser, s.first.name, s.first.id));
            
            i++;
        }
    }
}