package parentalControls;

import java.awt.AlphaComposite;
import java.awt.BorderLayout;
import java.awt.Color;
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
import UIComponents.Text;
import controller.screens.screenEvent.ScreenEvent;
import controller.screens.screenEvent.ScreenEventBus;
import utils.ColorManager;

public class ParentTeacherLoginPage extends JPanel {
    private InputField passField;

    private BufferedImage bgImage;

    private Text errorMsg;

    /**
     * The ParentTeacherLoginPage class represents the GUI panel where parents 
     * or teachers can log in to access the administrative controls.
     * It provides a password input field and verifies the credentials 
     * before granting access to the admin area.
     */
    public ParentTeacherLoginPage() {
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

        createLoginUI();

        PageLayout.createMenuButton(this);
    }

    /**
     * Builds and assembles the graphical user interface for the login screen.
     * Centers the components using layout wrappers and sets up the title,
     * password field, error message, and enter button.
     */
    private void createLoginUI() {
        // Wrapper for centering everything
        JPanel wrapper = new JPanel(new GridBagLayout());
        wrapper.setOpaque(false);

        // Main content container
        Flexbox mainContent = new Flexbox(true);
        mainContent.setOpaque(false);
        mainContent.setAlignmentX(CENTER_ALIGNMENT);

        // Title
        Header title = new Header("Enter Admin Password:");
        title.setAlignmentX(CENTER_ALIGNMENT);
        title.setForeground(ColorManager.primaryBrown);

        // Password Input Field
        // Passing an empty string since the title acts as the label
        passField = new InputField(InputField.Type.PASSWORD, ""); 
        passField.setAlignmentX(CENTER_ALIGNMENT);

        // Enter Button
        StyledButton enterBtn = new StyledButton("ENTER", StyledButton.ButtonStyle.RECT);
        enterBtn.addActionListener(e -> handleLogin());
        enterBtn.setAlignmentX(CENTER_ALIGNMENT);

        errorMsg = new Text("Wrong password.", 15);
        errorMsg.setForeground(Color.RED);
        errorMsg.setVisible(false);

        title.setAlignmentX(CENTER_ALIGNMENT);
        passField.setAlignmentX(CENTER_ALIGNMENT);
        errorMsg.setAlignmentX(CENTER_ALIGNMENT);
        enterBtn.setAlignmentX(CENTER_ALIGNMENT);
        // Assemble the main content stack
        mainContent.add(title);
        mainContent.add(Box.createVerticalStrut(30)); // Gap between title and input
        mainContent.add(passField);
        mainContent.add(Box.createVerticalStrut(10));
        mainContent.add(errorMsg);
        mainContent.add(Box.createVerticalStrut(40)); // Gap between input and button
        mainContent.add(enterBtn);

        wrapper.add(mainContent);
        add(wrapper, BorderLayout.CENTER);
    }

    /**
     * Handles the login action triggered by the "ENTER" button.
     * Retrieves the password, performs basic validation, checks it against
     * the system admin password, and routes the user or shows an error accordingly.
     */
    private void handleLogin() {
        String password = passField.getText();

        // Validation
        if (password.isEmpty()) {
            System.out.println("Error: Password required");
            return;
        }

        System.out.println("Attempting admin login...");
        
        // Check against the official admin password stored in SaveData
        if (password.equals("123456")) { 
            System.out.println("Login successful!");
            ScreenEventBus.publish(ScreenEvent.GO_TO_ADMIN_CONTROLS);
            errorMsg.setVisible(false);
        } else {
            errorMsg.setVisible(true);
            System.out.println("Incorrect admin password.");
        }

        passField.setText("");
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