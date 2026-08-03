package mainMenu;

import java.awt.BorderLayout;
import java.awt.Component;
import java.awt.Cursor;
import java.awt.Dimension;
import java.awt.Font;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.event.MouseAdapter;
import java.awt.event.MouseEvent;
import java.awt.image.BufferedImage;

import javax.imageio.ImageIO;
import javax.swing.BorderFactory;
import javax.swing.Box;
import javax.swing.BoxLayout;
import javax.swing.JLabel;
import javax.swing.JPanel;

import UIComponents.Header;
import UIComponents.Subheader;
import UIComponents.Text;
import controller.screens.screenEvent.ScreenEvent;
import controller.screens.screenEvent.ScreenEventBus;
import utils.ColorManager;
import utils.FontManager;

/**
 * Main Menu page, first screen the user sees when they open the game. 
 * Contains buttons to navigate to other pages, as well as credits.
 * 
 * This panel is displayed using a screen event, and the buttons publish screen events when clicked to navigate to other pages.
 * 
 * Extends JPanel and uses custom painting to display a background image.
 * 
 * @author Ali El-Rafih
 * @see controller.screens.screenEvent.ScreenEventBus
 */
public class MainMenuPage extends JPanel {

    /* Background image for the main menu */
    private BufferedImage bgImage;

    /*
     * Constructor for the main menu page.
     * Sets up the layout and loads the background image, 
     * as well as creating the buttons and credits.    
     */
    public MainMenuPage() {

        /* Set the layout for the main menu panel */
        setLayout(new BorderLayout());

        /* Load the background image
         * @exception If the image cannot be loaded, print an error message and stack trace.
         */
        try {
            bgImage = ImageIO.read(getClass().getResourceAsStream("/resources/images/islandBg.png"));
        } catch (Exception e) {
            System.err.println("Could not load background image!");
            e.printStackTrace();
        }

        /* Create the left and right panel and set positions*/
        JPanel leftPanel = createLeftPanel();
        JPanel rightPanel = new JPanel();
        rightPanel.setOpaque(false);
        // rightPanel.setBorder(new LineBorder(ColorManager.primaryBlue, 2));

        add(leftPanel, BorderLayout.WEST);
        add(rightPanel, BorderLayout.CENTER);
    }

    /*
     * Creates the left panel for the main menu.
     * @return The left panel.
     */
    private JPanel createLeftPanel() {
        JPanel leftPanel = new JPanel();
        leftPanel.setPreferredSize(new Dimension(320, 700));
        leftPanel.setBackground(ColorManager.primarySand);
        leftPanel.setLayout(new BorderLayout());

        /* Initialize fonts */
        Font bigTitleFont = getUiFont(64f, 64);
        Font menuFont = getUiFont(32f, 32);
        Font smallFont = getUiFont(15f, 15);

        JPanel content = new JPanel();
        content.setBackground(leftPanel.getBackground());
        content.setLayout(new BoxLayout(content, BoxLayout.Y_AXIS));
        content.setBorder(BorderFactory.createEmptyBorder(18, 22, 16, 16));

        /* Create the title for the main menu */
        Header title1 = new Header("Party");
        title1.setFont(bigTitleFont);
        title1.setForeground(ColorManager.primaryBlue);
        title1.setAlignmentX(Component.LEFT_ALIGNMENT);
        Header title2 = new Header("Islands");
        title2.setFont(bigTitleFont);
        title2.setForeground(ColorManager.primaryBlue);
        title2.setAlignmentX(Component.LEFT_ALIGNMENT);


        Subheader controls1 = new Subheader("Admin Controls");
        controls1.setFont(menuFont);
        styleLabel(controls1);
        controls1.setCursor(new Cursor(Cursor.HAND_CURSOR));

        

        /* Add mouse listeners to the controls subheaders to navigate to the admin login page when clicked, and change color on hover */
        controls1.addMouseListener(new MouseAdapter() {
            @Override
            public void mouseClicked(MouseEvent e) {
                ScreenEventBus.publish(ScreenEvent.GO_TO_ADMIN_LOGIN);
            }

            @Override
            public void mouseEntered(MouseEvent e) {
                controls1.setForeground(ColorManager.primaryBlue);
            }

            @Override
            public void mouseExited(MouseEvent e) {
                controls1.setForeground(ColorManager.primaryBrown);
            }
        });

        

        /* Create the instructions subheader */
        Subheader instructions = new Subheader("Instructions");
        instructions.setFont(menuFont);
        styleLabel(instructions);
        instructions.setCursor(new Cursor(Cursor.HAND_CURSOR));

        /* Add mouse listener to the instructions subheader */
        instructions.addMouseListener(new MouseAdapter() {
            @Override
            public void mouseClicked(MouseEvent e) {
                ScreenEventBus.publish(ScreenEvent.GO_TO_INSTRUCTIONS);
            }

            @Override
            public void mouseEntered(MouseEvent e) {
                instructions.setForeground(ColorManager.primaryBlue);
            }

            @Override
            public void mouseExited(MouseEvent e) {
                instructions.setForeground(ColorManager.primaryBrown);
            }
        });

        /* Create the high scores subheader */
        Subheader highScores = new Subheader("View High Scores");
        highScores.setFont(menuFont);
        styleLabel(highScores);
        highScores.setCursor(new Cursor(Cursor.HAND_CURSOR));

        /* Add mouse listener to the high scores subheader */
        highScores.addMouseListener(new MouseAdapter() {
            @Override
            public void mouseClicked(MouseEvent e) {
                ScreenEventBus.publish("HIGH_SCORES", new ViewHighScoresPage());
            }

            @Override
            public void mouseEntered(MouseEvent e) {
                highScores.setForeground(ColorManager.primaryBlue);
            }

            @Override
            public void mouseExited(MouseEvent e) {
                highScores.setForeground(ColorManager.primaryBrown);
            }
        });

        /* Create the login subheader */
        Subheader login = new Subheader("Login");
        login.setFont(menuFont);
        styleLabel(login);
        login.setCursor(new Cursor(Cursor.HAND_CURSOR));

        /* Add mouse listener to the login subheader */
        login.addMouseListener(new MouseAdapter() {
            @Override
            public void mouseClicked(MouseEvent e) {
                ScreenEventBus.publish(ScreenEvent.GO_TO_LOGIN);
            }

            @Override
            public void mouseEntered(MouseEvent e) {
                login.setForeground(ColorManager.primaryBlue);
            }

            @Override
            public void mouseExited(MouseEvent e) {
                login.setForeground(ColorManager.primaryBrown);
            }
        });

        /* Create the exit subheader */
        Subheader exit = new Subheader("Exit");
        exit.setFont(menuFont);
        styleLabel(exit);
        exit.setCursor(new Cursor(Cursor.HAND_CURSOR));

        /* Add mouse listener to the exit subheader to exit the game when clicked, and change color on hover */
        exit.addMouseListener(new MouseAdapter() {
            @Override
            public void mouseClicked(MouseEvent e) {
                System.exit(0);
            }

            @Override
            public void mouseEntered(MouseEvent e) {
                exit.setForeground(ColorManager.primaryBlue);
            }

            @Override
            public void mouseExited(MouseEvent e) {
                exit.setForeground(ColorManager.primaryBrown);
            }
        });

        /* Create the credits text */
        Text credits1Line1 = new Text("Ali El-Rafih, Arielle Tetelbaum, Lucas", 12f);
        credits1Line1.setFont(smallFont);
        credits1Line1.setForeground(ColorManager.secondaryBrown);
        credits1Line1.setAlignmentX(Component.LEFT_ALIGNMENT);

        Text credits1Line2 = new Text("Vanderwielen, Sanad Nassar, Sam Deitz", 12f);
        credits1Line2.setFont(smallFont);
        credits1Line2.setForeground(ColorManager.secondaryBrown);
        credits1Line2.setAlignmentX(Component.LEFT_ALIGNMENT);

        Text credits2Line1 = new Text("Team 44 - Winter 2026", 12f);
        credits2Line1.setFont(smallFont);
        credits2Line1.setForeground(ColorManager.secondaryBrown);
        credits2Line1.setAlignmentX(Component.LEFT_ALIGNMENT);
        credits2Line1.setBorder(BorderFactory.createEmptyBorder(8, 0, 0, 0));

        Text credits2Line2 = new Text("Created as part of CS2212 at Western", 12f);
        credits2Line2.setFont(smallFont);
        credits2Line2.setForeground(ColorManager.secondaryBrown);
        credits2Line2.setAlignmentX(Component.LEFT_ALIGNMENT);

        Text credits2Line3 = new Text("University", 12f);
        credits2Line3.setFont(smallFont);
        credits2Line3.setForeground(ColorManager.secondaryBrown);
        credits2Line3.setAlignmentX(Component.LEFT_ALIGNMENT);

        /* Add all components to the content panel with spacing in between */
        content.add(title1);
        content.add(title2);
        content.add(Box.createRigidArea(new Dimension(0, 26)));
        content.add(controls1);
        content.add(Box.createRigidArea(new Dimension(0, 16)));
        content.add(instructions);
        content.add(Box.createRigidArea(new Dimension(0, 10)));
        content.add(highScores);
        content.add(Box.createRigidArea(new Dimension(0, 10)));
        content.add(login);
        content.add(Box.createRigidArea(new Dimension(0, 10)));
        content.add(exit);
        content.add(Box.createVerticalGlue());
        content.add(credits1Line1);
        content.add(credits1Line2);
        content.add(credits2Line1);
        content.add(credits2Line2);
        content.add(credits2Line3);

        /* Add the content panel to the left panel */
        leftPanel.add(content, BorderLayout.CENTER);

        /* Return the left panel */
        return leftPanel;
    }

    /* Helper method to style the labels for the main menu */
    private void styleLabel(JLabel label) {
        label.setForeground(ColorManager.primaryBrown);
        label.setAlignmentX(Component.LEFT_ALIGNMENT);
    }

    /* Helper method to get the custom font for the main menu, with a fallback to a default font if the custom font cannot be loaded */
    private Font getUiFont(float preferredSize, int fallbackSize) {
        Font customFont = FontManager.getFont(preferredSize);
        if (customFont.getFamily().equalsIgnoreCase("SansSerif")) {
            return new Font("Serif", Font.PLAIN, fallbackSize);
        }
        return customFont;
    }

    /* Override the paintComponent method to draw the background image */
    @Override
    public void paintComponent(Graphics g) {
        super.paintComponent(g);

        /* Draw the background image if it was loaded successfully */
        if (bgImage != null) {
            Graphics2D g2d = (Graphics2D) g.create();
            // float opacity = 0.3f;
            // g2d.setComposite(AlphaComposite.getInstance(AlphaComposite.SRC_OVER, opacity));
            g2d.drawImage(bgImage, 0, 0, this.getWidth(), this.getHeight(), null);
            g2d.dispose();
        }
    }
}
