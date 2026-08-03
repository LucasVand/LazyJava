package mainMenu;

import java.awt.AlphaComposite;
import java.awt.BorderLayout;
import java.awt.Font;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.event.ComponentAdapter;
import java.awt.event.ComponentEvent;
import java.awt.image.BufferedImage;

import javax.imageio.ImageIO;
import javax.swing.JButton;
import javax.swing.JLabel;
import javax.swing.JPanel;
import javax.swing.JTextArea;
import javax.swing.border.EmptyBorder;

import UIComponents.Header;
import UIComponents.StyledButton;
import controller.screens.screenEvent.ScreenEvent;
import controller.screens.screenEvent.ScreenEventBus;
import utils.ColorManager;

/**
 * Instructions page, displayed when the user clicks on the "Instructions" button in the main menu.
 * Contains information about how to play the game and the points system.
 * 
 * This panel is displayed using a screen event
 * 
 * Extends JPanel and uses custom painting to display a background image.
 * 
 * @author Ali El-Rafih
 * @see mainMenu.MainMenuPage
 * @see controller.screens.screenEvent.ScreenEventBus
 */
public class InstructionsPage extends JPanel {

    /* Background image for the instructions page */
    private BufferedImage bgImage;

    /* Constructor for the instructions page, sets up the layout and components */
    public InstructionsPage() {

        /* Load the background image 
         * @exception If the image cannot be loaded, print an error message and stack trace.
         */
        try {
            bgImage = ImageIO.read(getClass().getResourceAsStream("/resources/images/islandBg.png"));
        } catch (Exception e) {
            System.err.println("Could not load background image!");
            e.printStackTrace();
        }

        
        setLayout(null);

        /* Define dimensions and positions for the main content panel */
        int panelWidth = 970;
        int panelHeight = 545;
        int panelX = 110;
        int panelY = 108;

        /* Create and style the header title for the instructions page */
        Header title = new Header("Instructions");
        title.setFont(title.getFont().deriveFont(44f));
        title.setForeground(ColorManager.primaryBrown);
        title.setBounds(panelX, panelY - 72, panelWidth, 55);
        add(title);

        /* Create the main content area for the instructions page, with a background color and null layout for absolute positioning of components */
        JPanel contentArea = new JPanel();
        contentArea.setBackground(ColorManager.primarySand);
        contentArea.setBounds(panelX, panelY, panelWidth, panelHeight);
        contentArea.setLayout(null);
        add(contentArea);

        /* Create and style the "How to Play" section of the instructions page, with a title and content area for the instructions text */
        JLabel howToPlay = new JLabel("How to Play:");
        howToPlay.setFont(title.getFont().deriveFont(32f));
        howToPlay.setForeground(ColorManager.primaryBlue);
        howToPlay.setBounds(20, 20, 200, 55);
        contentArea.add(howToPlay);

        /* Create the content box for the "How to Play" section, with a transparent background and a JTextArea for the instructions text */
        JPanel howToPlayContentBox = new JPanel();
        howToPlayContentBox.setBackground(contentArea.getBackground());
        howToPlayContentBox.setBounds(20, 80, panelWidth - 40, 145);
        howToPlayContentBox.setLayout(new BorderLayout());
        contentArea.add(howToPlayContentBox);

        /* Create and style the JTextArea for the "How to Play" instructions, with line wrapping and a transparent background */
        JTextArea howToPlayContent = new JTextArea(
                "Welcome to Party Islands! Log in or create an account. Once logged in, you can host a room or join someone else's room, play multiplayer or solo. In the lobby, once everyone selects ready, the game will start. Type your heart away, use power ups and work on your skills! Once you complete a level, or fail and lose all your lives, you will be directed to a game over screen, with your statistics and race leaderboard displayed. You can leave a game at any time or play again!"
        );
        howToPlayContent.setFont(new Font("SansSerif", Font.PLAIN, 18));
        howToPlayContent.setEditable(false);
        howToPlayContent.setFocusable(false);
        howToPlayContent.setLineWrap(true);
        howToPlayContent.setWrapStyleWord(true);
        howToPlayContent.setOpaque(false);
        howToPlayContent.setBorder(new EmptyBorder(0, 0, 0, 0));
        howToPlayContent.setForeground(ColorManager.primaryBrown);
        howToPlayContentBox.add(howToPlayContent, BorderLayout.CENTER);

        /* Create and style the "Points System" section of the instructions page, with a title and content area for the points system text */
        JLabel pointsSystem = new JLabel("Points System:");
        pointsSystem.setFont(title.getFont().deriveFont(32f));
        pointsSystem.setForeground(ColorManager.primaryBlue);
        pointsSystem.setBounds(20, 225, 320, 55);
        contentArea.add(pointsSystem);

        /* Create the content box for the "Points System" section, with a transparent background and a JTextArea for the points system text */
        JPanel pointsSystemContentBox = new JPanel();
        pointsSystemContentBox.setBackground(contentArea.getBackground());
        pointsSystemContentBox.setBounds(20, 285, panelWidth - 40, 75);
        pointsSystemContentBox.setLayout(new BorderLayout());
        contentArea.add(pointsSystemContentBox);

        /* Create and style the JTextArea for the "Points System" instructions, with line wrapping and a transparent background */
        JTextArea pointsSystemContent = new JTextArea(
                "Every correct word earns points and builds your momentum. The faster and more accurately you type, the better your final score will be. Missing words, making mistakes, or losing lives will hurt your total, so stay focused and keep the streak alive for as long as possible."
        );
        pointsSystemContent.setFont(new Font("SansSerif", Font.PLAIN, 18));
        pointsSystemContent.setEditable(false);
        pointsSystemContent.setFocusable(false);
        pointsSystemContent.setLineWrap(true);
        pointsSystemContent.setWrapStyleWord(true);
        pointsSystemContent.setOpaque(false);
        pointsSystemContent.setBorder(new EmptyBorder(0, 0, 0, 0));
        pointsSystemContent.setForeground(ColorManager.primaryBrown);
        pointsSystemContentBox.add(pointsSystemContent, BorderLayout.CENTER);

        /* Create and style the "Powerups" section of the instructions page, with a title and content area for the powerups text */
        JLabel powerups = new JLabel("Powerups:");
        powerups.setFont(title.getFont().deriveFont(32f));
        powerups.setBounds(20, 375, 220, 55);
        powerups.setForeground(ColorManager.primaryBlue);
        contentArea.add(powerups);

        /* Create the content box for the "Powerups" section, with a transparent background and a JTextArea for the powerups text */
        JPanel powerupsContentBox = new JPanel();
        powerupsContentBox.setBackground(contentArea.getBackground());
        powerupsContentBox.setBounds(20, 430, panelWidth - 40, 95);
        powerupsContentBox.setLayout(new BorderLayout());
        contentArea.add(powerupsContentBox);

        /* Create and style the JTextArea for the "Powerups" instructions, with line wrapping and a transparent background */
        JTextArea powerupsContent = new JTextArea(
            "Boost: Increase your score for a short time. Type quickly and accurately to earn double points.\n" +
            "+20 Health: Adds an extra 20 health points to help you complete the level.\n" +
            "Skip Word: Skip a difficult word without losing hearts or points."
        );
        powerupsContent.setFont(new Font("SansSerif", Font.PLAIN, 18));
        powerupsContent.setEditable(false);
        powerupsContent.setFocusable(false);
        powerupsContent.setLineWrap(true);
        powerupsContent.setWrapStyleWord(true);
        powerupsContent.setOpaque(false);
        powerupsContent.setBorder(new EmptyBorder(0, 0, 0, 0));
        powerupsContent.setForeground(ColorManager.primaryBrown);
        powerupsContentBox.add(powerupsContent, BorderLayout.CENTER);

        /* Create and style the "Main Menu" button for the instructions page, with an action listener to navigate back to the main menu when clicked */
        StyledButton mainMenuButton = new StyledButton("Main Menu", StyledButton.ButtonStyle.ROUNDED_RECT);
        mainMenuButton.addActionListener(e -> {
            ScreenEventBus.publish(ScreenEvent.GO_TO_MAIN_MENU);
        });
        int buttonWidth = 190;
        int buttonHeight = 60;
        mainMenuButton.setBounds(0, 0, buttonWidth, buttonHeight);
        add(mainMenuButton);
        setComponentZOrder(mainMenuButton, 0);
        positionMainMenuButton(mainMenuButton, buttonWidth, buttonHeight);

        /* Add a component listener to reposition the main menu button when the instructions page is resized */
        addComponentListener(new ComponentAdapter() {
            @Override
            public void componentResized(ComponentEvent e) {
                positionMainMenuButton(mainMenuButton, buttonWidth, buttonHeight);
            }
        });
    }

    /* Helper method to position the main menu button in the bottom right corner of the instructions page, with margins from the edges */
    private void positionMainMenuButton(JButton mainMenuButton, int buttonWidth, int buttonHeight) {
        int rightMargin = 24;
        int bottomMargin = 18;
        int buttonX = getWidth() - buttonWidth - rightMargin;
        int buttonY = getHeight() - buttonHeight - bottomMargin;
        mainMenuButton.setBounds(buttonX, buttonY, buttonWidth, buttonHeight);
    }

    /* Override the paintComponent method to draw the background image with a specified opacity, scaling it to fill the entire panel */
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
