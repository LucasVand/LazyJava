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
 * The ResetPasswordPage class represents a GUI panel that allows a user 
 * to change the password for a specific account.
 * It requires the user to input their current (old) password for verification 
 * before allowing them to set and save a new password.
 */
public class ResetPasswordPage extends JPanel {

    /** The input field for the user to enter their current password and their new desired password. */
    private InputField oldPassField, newPassField;
    private Account targetAccount;
    private BufferedImage bgImage;

    /**
     * Constructs the ResetPasswordPage for a specific target account.
     * Initializes the panel, loads the background image, sets up the form UI, 
     * and adds a back navigation button.
     *
     * @param account The Account object representing the user whose password will be reset.
     */
    public ResetPasswordPage(Account account) {
        this.targetAccount = account;
        
        setVisible(true);
        setPreferredSize(new Dimension(1200, 700));
        setLayout(new BorderLayout());
        try {
            bgImage = ImageIO.read(getClass().getResourceAsStream("/resources/images/islandBg.png"));
        } catch (Exception e) {
            System.err.println("Could not load background image!");
            e.printStackTrace();
        }

        createResetUI();
        PageLayout.createBackButton(this, e -> {
            ScreenEventBus.publish(ScreenEvent.GO_TO_ADMIN_CONTROLS);
        });
    }

    /**
     * Builds and assembles the graphical user interface for the password reset form.
     * This includes the target username display, input fields for old and new passwords, 
     * and the confirmation button.
     */
    private void createResetUI() {
        JPanel wrapper = new JPanel(new GridBagLayout());
        wrapper.setOpaque(false);

        Flexbox mainContent = new Flexbox(true);
        mainContent.setOpaque(false);

        Header title = new Header("RESET PASSWORD");
        title.setForeground(ColorManager.primaryBrown);
        
        // Gray form box
        Flexbox form = new Flexbox(true);
        form.setPreferredSize(new Dimension(700, 500));
        form.setBackground(ColorManager.primarySand);
        form.addPadding(40);

        // Display current target username
        Header userDisplay = new Header("Username: " + targetAccount.getIdentity().getUsername());
        userDisplay.setAlignmentX(CENTER_ALIGNMENT);
        userDisplay.setForeground(ColorManager.primaryBrown);

        
        oldPassField = new InputField(20, InputField.Type.PASSWORD, "Old Password:");
        oldPassField.setAlignmentX(CENTER_ALIGNMENT);


        newPassField = new InputField(20, InputField.Type.PASSWORD, "New Password:");
        newPassField.setAlignmentX(CENTER_ALIGNMENT);



        StyledButton resetBtn = new StyledButton("RESET", StyledButton.ButtonStyle.RECT);
        resetBtn.setAlignmentX(CENTER_ALIGNMENT);
        resetBtn.addActionListener(e -> handleReset());

        // Assemble the form elements with vertical spacing
        form.add(userDisplay);
        form.add(Box.createVerticalStrut(20));
        form.add(oldPassField);
        form.add(Box.createVerticalStrut(20));
        form.add(newPassField);
        form.add(Box.createVerticalStrut(40));
        form.add(resetBtn);

        mainContent.add(title);
        mainContent.add(Box.createVerticalStrut(20));
        mainContent.add(form);
        wrapper.add(mainContent);
        add(wrapper, BorderLayout.CENTER);
    }

    /**
     * Handles the password reset logic triggered by the "RESET" button.
     * Verifies that the entered old password matches the account's current password
     * before applying and saving the new password.
     */
    private void handleReset() {
        String oldP = oldPassField.getText();
        String newP = newPassField.getText();

        // 1. Verify old password (Logic depends on your Identity.java methods)
        if (targetAccount.getIdentity().getPassword().equals(oldP)) {
            // 2. Update password 
            // NOTE: You may need to add a setPassword() method to Identity.java if it doesn't exist!
            targetAccount.getIdentity().setPassword(newP); 
            SaveData.save();
            
            System.out.println("Password successfully updated!");
            ScreenEventBus.publish(ScreenEvent.GO_TO_ADMIN_CONTROLS);
        } else {
            System.out.println("Error: Old password does not match.");
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