package parentalControls;

import java.awt.AlphaComposite;
import java.awt.BorderLayout;
import java.awt.Dimension;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.GridBagConstraints;
import java.awt.GridBagLayout;
import java.awt.Insets;
import java.awt.image.BufferedImage;

import javax.imageio.ImageIO;
import javax.swing.Box;
import javax.swing.JPanel;

import UIComponents.Flexbox;
import UIComponents.Header;
import UIComponents.InputField;
import UIComponents.PageLayout;
import UIComponents.StyledButton;
import controller.screens.screenEvent.ScreenEvent;
import controller.screens.screenEvent.ScreenEventBus;
import saveData.Account;
import saveData.SaveData;
import utils.ColorManager;

/**
 * The TeacherControlsPage class represents the main administrative dashboard.
 * It provides teachers or parents with tools to search for specific players, 
 * view their statistics, manage their accounts (create, delete, reset passwords), 
 * and clear global or individual statistics.
 */
public class TeacherControlsPage extends JPanel {
    private InputField searchField;

    // --- UPDATED CONSTRUCTOR TO ACCEPT CONTROLLER ---
    private BufferedImage bgImage;

    /**
     * Constructs the TeacherControlsPage.
     * Initializes panel dimensions, loads the background image, sets up the 
     * dashboard user interface, and adds a navigation button to return to the menu.
     */
    public TeacherControlsPage() {

        setVisible(true);
        setPreferredSize(new Dimension(1200, 700));
        setFocusable(true);
        setLayout(new BorderLayout());
        try {
            bgImage = ImageIO.read(getClass().getResourceAsStream("/resources/images/islandBg.png"));
        } catch (Exception e) {
            System.err.println("Could not load background image!");
            e.printStackTrace();
        }

        createDashboardUI();

        PageLayout.createMenuButton(this);
    }

    /**
     * Builds and assembles the graphical user interface for the admin dashboard.
     * Utilizes a GridBagLayout to organize a search bar, account management buttons,
     * and global statistic reset tools into structured columns.
     */
    private void createDashboardUI() {
        JPanel wrapper = new JPanel(new GridBagLayout());
        wrapper.setOpaque(false);

        Flexbox mainContent = new Flexbox(true);
        mainContent.setOpaque(false);
        mainContent.setAlignmentX(CENTER_ALIGNMENT);

        Header title = new Header("ADMIN CONTROLS");
        title.setAlignmentX(CENTER_ALIGNMENT);
        title.setForeground(ColorManager.primaryBrown);

        JPanel dashboardBox = new JPanel(new GridBagLayout());
        dashboardBox.setPreferredSize(new Dimension(850, 600));
        dashboardBox.setBackground(ColorManager.primarySand); // Light gray

        GridBagConstraints gbc = new GridBagConstraints();
        gbc.insets = new Insets(10, 20, 10, 20);

        // Row 0: Search
        gbc.gridx = 0;
        gbc.gridy = 0;
        gbc.anchor = GridBagConstraints.EAST;
        Header searchLabel = new Header("Search for player:");
        searchLabel.setForeground(ColorManager.primaryBlue);
        dashboardBox.add(searchLabel, gbc);

        gbc.gridx = 1;
        gbc.gridy = 0;
        gbc.anchor = GridBagConstraints.WEST;
        searchField = new InputField(InputField.Type.BASIC, "");
        dashboardBox.add(searchField, gbc);

        // --- RIGHT COLUMN PILL BUTTONS ---
        gbc.gridx = 1;
        gbc.anchor = GridBagConstraints.CENTER;
        gbc.fill = GridBagConstraints.HORIZONTAL;

        // 1. View Stats
        gbc.gridy = 1;
        StyledButton viewStatsBtn = new StyledButton("View Player Statistics", StyledButton.ButtonStyle.ROUNDED_RECT);
        viewStatsBtn.addActionListener(e -> {
            Account foundAcc = findAccountByUsername(searchField.getText());
            if (foundAcc != null) {
                ScreenEventBus.publish("PLAYER_STATS",
                        new PlayerStatisticsPage(foundAcc.getIdentity().getUsername(), foundAcc.getStats()));
            } else {
                System.out.println("User not found.");
            }
        });
        dashboardBox.add(viewStatsBtn, gbc);

        // 2. Reset Password
        gbc.gridy = 2;
        StyledButton resetPassBtn = new StyledButton("Reset Password", StyledButton.ButtonStyle.ROUNDED_RECT);
        resetPassBtn.addActionListener(e -> {
            Account foundAcc = findAccountByUsername(searchField.getText());
            if (foundAcc != null) {
                ScreenEventBus.publish("RESET_PASS", new ResetPasswordPage(foundAcc));
            } else {
                System.out.println("User not found.");
            }
        });
        dashboardBox.add(resetPassBtn, gbc);

        // 3. Reset Statistics
        gbc.gridy = 3;
        StyledButton resetStatsBtn = new StyledButton("Reset Statistics", StyledButton.ButtonStyle.ROUNDED_RECT);
        resetStatsBtn.addActionListener(e -> {
            Account foundAcc = findAccountByUsername(searchField.getText());
            if (foundAcc != null) {
                foundAcc.resetStats();
                SaveData.save();
                System.out.println("Stats reset!");
                searchField.setText("");
            } else {
                System.out.println("User not found.");
            }
        });
        dashboardBox.add(resetStatsBtn, gbc);

        // --- LEFT COLUMN RECT BUTTONS ---
        gbc.gridx = 0;
        gbc.fill = GridBagConstraints.NONE;
        gbc.anchor = GridBagConstraints.WEST;

        // 4. Create Account
        gbc.gridy = 4;
        gbc.insets = new Insets(40, 20, 10, 20);
        StyledButton createAccBtn = new StyledButton("Create an Account", StyledButton.ButtonStyle.RECT);
        createAccBtn.addActionListener(e -> {
            ScreenEventBus.publish(ScreenEvent.GO_TO_CREATE_ACCOUNT);
        });
        dashboardBox.add(createAccBtn, gbc);

        // 5. Delete Account (Added from earlier!)
        gbc.gridy = 5;
        gbc.insets = new Insets(10, 20, 10, 20);
        StyledButton deleteAccBtn = new StyledButton("Delete Account", StyledButton.ButtonStyle.RECT);
        deleteAccBtn.addActionListener(e -> {
            Account foundAcc = findAccountByUsername(searchField.getText());
            if (foundAcc != null) {
                SaveData.getData().deleteAccount(foundAcc);
                SaveData.save();
                System.out.println("Account deleted!");
                searchField.setText("");
            } else {
                System.out.println("User not found.");
            }
        });
        dashboardBox.add(deleteAccBtn, gbc);

        // 6. Reset High Scores
        gbc.gridy = 6;
        StyledButton resetScoreBtn = new StyledButton("Reset High Score Table", StyledButton.ButtonStyle.RECT);
        resetScoreBtn.addActionListener(e -> {
            SaveData.getData().getHighscoreTable().reset();
            SaveData.save();
            System.out.println("Resetting High Scores...");
        });
        dashboardBox.add(resetScoreBtn, gbc);

        mainContent.add(title);
        mainContent.add(Box.createVerticalStrut(20));
        mainContent.add(dashboardBox);

        wrapper.add(mainContent);
        add(wrapper, BorderLayout.CENTER);
    }

    /**
     * Helper method to search the saved data for an account matching the provided username.
     * * @param username The exact username to search for.
     * @return The matching Account object if found, or null if no match exists or the input is empty.
     */
    private Account findAccountByUsername(String username) {
        if (username.isEmpty())
            return null;
        for (Account a : SaveData.getData().getAccounts()) {
            if (a.getIdentity().getUsername().equals(username)) {
                return a;
            }
        }
        return null;
    }

    /**
     * Custom painting method to render the background image with a specific opacity.
     * * @param g The Graphics object used for drawing operations.
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
