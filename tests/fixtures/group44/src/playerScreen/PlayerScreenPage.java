package playerScreen;

import java.awt.BorderLayout;
import java.awt.Color;
import java.awt.Component;
import java.awt.Cursor;
import java.awt.Dimension;
import java.awt.Font;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.GridLayout;
import java.awt.MouseInfo;
import java.awt.Point;
import java.awt.PointerInfo;
import java.awt.Rectangle;
import java.awt.event.MouseAdapter;
import java.awt.event.MouseEvent;
import java.awt.image.BufferedImage;
import java.util.TimerTask;

import javax.imageio.ImageIO;
import javax.swing.BorderFactory;
import javax.swing.Box;
import javax.swing.BoxLayout;
import javax.swing.JButton;
import javax.swing.JLabel;
import javax.swing.JPanel;
import javax.swing.SwingConstants;
import javax.swing.SwingUtilities;

import UIComponents.Header;
import UIComponents.Subheader;
import broadcasting.RoomServer;
import controller.screens.screenEvent.ScreenEvent;
import controller.screens.screenEvent.ScreenEventBus;
import gameplayScreens.MainGame;
import saveData.Account;
import saveData.SaveData;
import utils.ColorManager;
import utils.FontManager;

/**
 * Player screen page, displayed when the user is in the player screen.
 * 
 * This panel is displayed using a screen event
 * 
 * Extends JPanel and uses custom painting to display a background image.
 * 
 * @author Ali El-Rafih
 */
public class PlayerScreenPage extends JPanel {

    /* Background image for the player screen */
    private BufferedImage bgImage;

    /* Reference to the currently logged in account, used to display player information and manage statistics */
    private Account account;

    /* Constructor for the PlayerScreenPage, initializes the layout, loads the background image, and sets up the left and right panels */
    private Subheader selectLevel;
    private int selectedLevel = 0;
    private String[] levels = {"Select Level", "Level 1", "Level 2", "Level 3"};

    public PlayerScreenPage() {
        
        /* Get the currently logged in account from the save data, which will be used to display player information and manage statistics */
        account = SaveData.getData().getLoggedInAccount();
        setLayout(new BorderLayout());

        /* Load the background image for the player screen, which will be drawn in the paintComponent method to create a visually appealing backdrop for the player information and menu options */
        try {
            bgImage = ImageIO.read(getClass().getResourceAsStream("/resources/images/islandBg.png"));
        } catch (Exception e) {
            System.err.println("Could not load background image!");
            e.printStackTrace();
        }
        

        /* Create the left and right panel and set positions*/
        JPanel leftPanel = createLeftPanel();
        JPanel rightPanel = new PlayerPlaceholder(account);
        // rightPanel.setBorder(new LineBorder(ColorManager.primaryBlue, 2));

        add(leftPanel, BorderLayout.WEST);
        add(rightPanel, BorderLayout.CENTER);
    }

    /* Helper method to create the left panel of the player screen, which contains the menu options and player information */
    private void updateLevelSelect() {
        selectedLevel = (selectedLevel+1) % levels.length;
        selectLevel.setText(levels[selectedLevel]);
        selectLevel.revalidate();
        selectLevel.repaint();
    }

    private JPanel createLeftPanel() {
        JPanel leftPanel = new JPanel();
        leftPanel.setPreferredSize(new Dimension(320, 700));
        leftPanel.setBackground(ColorManager.primarySand);
        leftPanel.setLayout(new BorderLayout());

        /* Initialize fonts */
        Font bigTitleFont = getUiFont(64f, 64);
        Font menuFont = getUiFont(32f, 32);

        /* Create the content panel for the left side, which will hold the menu options and player information, and set its layout and styling */
        JPanel content = new JPanel();
        content.setBackground(leftPanel.getBackground());
        content.setLayout(new BoxLayout(content, BoxLayout.Y_AXIS));
        content.setBorder(BorderFactory.createEmptyBorder(18, 22, 16, 16));

        /* Create and style the header labels for the "Party" and "Islands" sections of the left panel, which will be displayed at the top of the menu options */
        Header title1 = new Header("Party");
        title1.setFont(bigTitleFont);
        title1.setForeground(ColorManager.primaryBlue);
        title1.setAlignmentX(Component.LEFT_ALIGNMENT);
        Header title2 = new Header("Islands");
        title2.setFont(bigTitleFont);
        title2.setForeground(ColorManager.primaryBlue);
        title2.setAlignmentX(Component.LEFT_ALIGNMENT);

        /* Create and style the "Play New Game" menu option, which will display a dropdown menu when hovered over, allowing the player to choose between joining or hosting a game */
        Subheader playNewGame = new Subheader("Play New Game");
        playNewGame.setFont(menuFont);
        playNewGame.setCursor(new Cursor(Cursor.HAND_CURSOR));
        styleLabel(playNewGame);

        /* Create and style the "Select Level" menu option, which will navigate the player to the instructions page when clicked, allowing them to choose a level to play */
        selectLevel = new Subheader("Select Level");
        selectLevel.setFont(menuFont);

        /* Add a mouse listener to the "Select Level" menu option to handle clicks and hover effects, changing the text color when hovered and navigating to the instructions page when clicked */
        selectLevel.addMouseListener(new MouseAdapter() {
            @Override
            public void mouseClicked(MouseEvent e) {
                updateLevelSelect();
            }

            @Override
            public void mouseEntered(MouseEvent e) {
                selectLevel.setForeground(ColorManager.primaryBlue);
            }

            @Override
            public void mouseExited(MouseEvent e) {
                selectLevel.setForeground(ColorManager.primaryBrown);
            }
        });
        styleLabel(selectLevel);

        /* Create the dropdown panel for the "Play New Game" menu option, which will contain buttons for joining or hosting a game, and set its layout and styling */   
        JPanel dropdownPanel = new JPanel();
        dropdownPanel.setLayout(new GridLayout(2, 1));
        dropdownPanel.setBackground(new Color(225, 225, 225));
        dropdownPanel.setBorder(BorderFactory.createLineBorder(new Color(180, 180, 180)));
        dropdownPanel.setMaximumSize(new Dimension(220, 90));
        dropdownPanel.setPreferredSize(new Dimension(220, 90));
        dropdownPanel.setVisible(false);
        dropdownPanel.setAlignmentX(Component.LEFT_ALIGNMENT);

        /* Create and style the "Join" and "Host" buttons for the dropdown menu, which will allow the player to either join an existing game or host a new game, with hover effects to change their appearance when hovered over */
        JButton joinButton = new JButton("Join");
        joinButton.setFont(getUiFont(24f, 24));
        joinButton.setFocusPainted(false);
        joinButton.setBorderPainted(false);
        joinButton.setBackground(ColorManager.secondaryBrown);
        joinButton.setForeground(ColorManager.primarySand);
        joinButton.setHorizontalAlignment(SwingConstants.LEFT);
        joinButton.setCursor(new Cursor(Cursor.HAND_CURSOR));

        JButton hostButton = new JButton("Host");
        hostButton.setFont(getUiFont(24f, 24));
        hostButton.setFocusPainted(false);
        hostButton.setBorderPainted(false);
        hostButton.setBackground(ColorManager.secondaryBrown);
        hostButton.setForeground(ColorManager.primarySand);
        hostButton.setHorizontalAlignment(SwingConstants.LEFT);
        hostButton.setCursor(new Cursor(Cursor.HAND_CURSOR));

        /* Add mouse listeners to the "Join" and "Host" buttons to handle clicks and hover effects, changing their appearance when hovered and navigating to the appropriate screens when clicked */
        joinButton.addMouseListener(new MouseAdapter() {
            @Override
            public void mouseEntered(MouseEvent e) {
                joinButton.setBackground(ColorManager.primaryBlue);
                joinButton.setForeground(Color.WHITE);
            }

            @Override
            public void mouseExited(MouseEvent e) {
                joinButton.setBackground(ColorManager.secondaryBrown);
                joinButton.setForeground(ColorManager.primarySand);
            }
        });
        joinButton.addActionListener(e -> ScreenEventBus.publish("OPEN", new FindRoomsPage()));

        hostButton.addMouseListener(new MouseAdapter() {
            @Override
            public void mouseEntered(MouseEvent e) {
                hostButton.setBackground(ColorManager.primaryBlue);
                hostButton.setForeground(Color.WHITE);
            }

            @Override
            public void mouseExited(MouseEvent e) {
                hostButton.setBackground(ColorManager.secondaryBrown);
                hostButton.setForeground(ColorManager.primarySand);
            }
        });
        hostButton.addActionListener(e -> {
            RoomServer server = new RoomServer(SaveData.getData().getLoggedInAccount().getIdentity().getUsername());
            server.startBroadcasting();
            new MainGame("localhost", true, selectedLevel == 0 ? 1 : selectedLevel, server);
        });

        dropdownPanel.add(joinButton);
        dropdownPanel.add(hostButton);

        MouseAdapter dropdownHoverHandler = new MouseAdapter() {
            private void updateDropdownVisibility() {
                PointerInfo pointerInfo = MouseInfo.getPointerInfo();
                if (pointerInfo == null) {
                    return;
                }

                Point mouseLocation = pointerInfo.getLocation();
                SwingUtilities.convertPointFromScreen(mouseLocation, content);

                Rectangle triggerBounds = SwingUtilities.convertRectangle(
                        playNewGame.getParent(),
                        playNewGame.getBounds(),
                        content);
                Rectangle dropdownBounds = SwingUtilities.convertRectangle(
                        dropdownPanel.getParent(),
                        dropdownPanel.getBounds(),
                        content);

                boolean overTrigger = triggerBounds.contains(mouseLocation);
                boolean overDropdown = dropdownBounds.contains(mouseLocation);
                boolean shouldShow = overTrigger || overDropdown;

                dropdownPanel.setVisible(shouldShow);
                playNewGame.setForeground(shouldShow ? ColorManager.primaryBlue : ColorManager.primaryBrown);
                content.revalidate();
                content.repaint();
            }

            @Override
            public void mouseEntered(MouseEvent e) {
                dropdownPanel.setVisible(true);
                playNewGame.setForeground(ColorManager.primaryBlue);
                content.revalidate();
                content.repaint();
            }

            @Override
            public void mouseExited(MouseEvent e) {
                new java.util.Timer().schedule(new TimerTask() {
                    @Override
                    public void run() {
                        updateDropdownVisibility();
                    }
                }, 300);
            }
        };

        playNewGame.addMouseListener(dropdownHoverHandler);
        dropdownPanel.addMouseListener(dropdownHoverHandler);

        /* Create and style the "Logout" menu option, which will log the player out and navigate back to the main menu when clicked, with hover effects to change its appearance when hovered over */
        Subheader logout = new Subheader("Logout");
        logout.setFont(menuFont);

        /* Add a mouse listener to the "Logout" menu option to handle clicks and hover effects, changing the text color when hovered and logging the player out and navigating back to the main menu when clicked */
        logout.addMouseListener(new MouseAdapter() {
            @Override
            public void mouseClicked(MouseEvent e) {
                SaveData.getData().logoutAccount();
                ScreenEventBus.publish(ScreenEvent.GO_TO_MAIN_MENU);
            }

            @Override
            public void mouseEntered(MouseEvent e) {
                logout.setForeground(ColorManager.primaryBlue);
            }

            @Override
            public void mouseExited(MouseEvent e) {
                logout.setForeground(ColorManager.primaryBrown);
            }
        });
        styleLabel(logout);

        /* Add all the components to the content panel in the correct order, with spacing between them using rigid areas and vertical glue to push the logout option to the bottom */
        content.add(title1);
        content.add(title2);
        content.add(Box.createRigidArea(new Dimension(0, 28)));
        content.add(playNewGame);
        content.add(Box.createRigidArea(new Dimension(0, 6)));
        content.add(dropdownPanel);
        content.add(Box.createRigidArea(new Dimension(0, 10)));
        content.add(selectLevel);
        content.add(Box.createVerticalGlue());
        content.add(logout);

        /* Add the content panel to the left panel and return it */
        leftPanel.add(content, BorderLayout.CENTER);
        return leftPanel;
    }

    /* Helper method to style the menu option labels consistently, setting their foreground color and alignment */
    private void styleLabel(JLabel label) {
        label.setForeground(ColorManager.primaryBrown);
        label.setAlignmentX(Component.LEFT_ALIGNMENT);
    }

    /* Helper method to get a custom font for the UI, with a fallback to a standard font if the custom font cannot be loaded, ensuring that the UI remains visually consistent even if there are issues with loading the custom font */
    private Font getUiFont(float preferredSize, int fallbackSize) {
        Font customFont = FontManager.getFont(preferredSize);
        if (customFont.getFamily().equalsIgnoreCase("SansSerif")) {
            return new Font("Serif", Font.PLAIN, fallbackSize);
        }
        return customFont;
    }

    /* Override the paintComponent method to draw the background image with a specified opacity, scaling it to fill the entire panel, creating a visually appealing backdrop for the player information and menu options */
    @Override
    public void paintComponent(Graphics g) {
        super.paintComponent(g);

        if (bgImage != null) {
            Graphics2D g2d = (Graphics2D) g.create();
            // float opacity = 0.3f;
            // g2d.setComposite(AlphaComposite.getInstance(AlphaComposite.SRC_OVER,
            // opacity));
            g2d.drawImage(bgImage, 0, 0, this.getWidth(), this.getHeight(), null);
            g2d.dispose();
        }
    }
}
