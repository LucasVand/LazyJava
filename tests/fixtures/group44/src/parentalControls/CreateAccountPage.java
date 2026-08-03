package parentalControls;

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
 * The CreateAccountPage class represents a GUI panel that allows a user 
 * to create a new administrative or parental control account.
 * It features a form for username and password entry and handles
 * basic input validation before saving the account data.
 */
public class CreateAccountPage extends JPanel {
    /** The chosen username and password for the new account. */
    private String username, password;
    /** The text input field for the user to enter their username and password. */
    InputField userField, passField;
    private BufferedImage bgImage;

    /**
     * Constructs the CreateAccountPage.
     * Initializes the panel's properties, loads the background image, 
     * builds the signup form UI, and sets up the back navigation button.
     */
    public CreateAccountPage() {
        setVisible(true);
        setPreferredSize(new Dimension(1200, 700));
        setFocusable(true);
        setLayout(new BorderLayout());
        // Try to load the background image from the resources folder catch error
        try {
            // Start with a "/" to look from the root of the JAR, and use getResourceAsStream
            bgImage = ImageIO.read(getClass().getResourceAsStream("/resources/images/islandBg.png"));
        } catch (Exception e) {
            System.err.println("Could not load background image!");
            e.printStackTrace();
        }

        // Initialize and add the form components to the panel
        createSignupForm();
        PageLayout.createBackButton(this, e -> {
            ScreenEventBus.publish(ScreenEvent.GO_TO_ADMIN_CONTROLS);
        }
        );
    }

    /**
     * Builds and assembles the graphical user interface for the sign-up form.
     * This includes the title, input fields, creation button, and all associated layout managers.
     */
    private void createSignupForm() {
        // Wrapper for centering everything
        JPanel wrapper = new JPanel(new GridBagLayout());
        wrapper.setOpaque(false);

        // Main content container (holds title + gray form box)
        Flexbox mainContent = new Flexbox(true);
        mainContent.setOpaque(false);
        mainContent.setAlignmentX(CENTER_ALIGNMENT);

        // Title positioned outside and above the gray box
        Header title = new Header("CREATE ACCOUNT");
        title.setAlignmentX(CENTER_ALIGNMENT);
        title.setForeground(ColorManager.primaryBrown);

        // Gray background form box
        Flexbox form = new Flexbox(true);
        form.setPreferredSize(new Dimension(650, 400));
        form.setAlignmentX(CENTER_ALIGNMENT);
        form.setBackground(ColorManager.primarySand); // Light gray
        form.addPadding(50);

        // Input field for username
        userField = new InputField(InputField.Type.BASIC, "Username:");
        userField.setAlignmentX(CENTER_ALIGNMENT);

        // Input field for password
        passField = new InputField(InputField.Type.PASSWORD, "Password:");
        passField.setAlignmentX(CENTER_ALIGNMENT);

        // Create button inside the gray box
        StyledButton createBtn = new StyledButton("CREATE", StyledButton.ButtonStyle.RECT);
        createBtn.addActionListener(e -> handleCreateAccount());
        createBtn.setAlignmentX(CENTER_ALIGNMENT);

        // Assemble the gray box layout
        form.add(Box.createVerticalGlue());
        form.add(userField);
        form.add(Box.createVerticalStrut(30));
        form.add(passField);
        form.add(Box.createVerticalStrut(50));
        form.add(createBtn);
        form.add(Box.createVerticalGlue());

        // Assemble the main content stack
        mainContent.add(title);
        mainContent.add(Box.createVerticalStrut(20)); // Gap between title and gray box
        mainContent.add(form);

        wrapper.add(mainContent);
        add(wrapper, BorderLayout.CENTER);
    }

    private void handleCreateAccount() {
        username = userField.getText();
        password = passField.getText();

        // Validation
        if (username.isEmpty() || password.isEmpty()) {
            System.out.println("Error: All fields required");
            return;
        }

        if (password.length() < 4) {
            System.out.println("Error: Password must be at least 4 characters");
            return;
        }

        try {
            Account newAccount = new Account(username, password);
            SaveData.getData().createAccount(newAccount);
            SaveData.save(); // Saves to data.bin

            System.out.println("Account created successfully for: " + username);

            // Automatically route them to the login page so they can log in!
            ScreenEventBus.publish(ScreenEvent.GO_TO_ADMIN_CONTROLS);

        } catch (Exception e) {
            System.out.println("Error creating account: " + e.getMessage());
        }
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
