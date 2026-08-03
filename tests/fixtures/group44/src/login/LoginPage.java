package login;

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
import controller.screens.screenEvent.ScreenEventBus;
import playerScreen.PlayerScreenPage;
import saveData.SaveData;
import utils.ColorManager;

public class LoginPage extends JPanel {
    private String username = "", password = "";
    InputField userField, passField;
    private BufferedImage bgImage;
    private Text errorMsg = new Text("Wrong username or password.", 15);

    public LoginPage() {
        setVisible(true);
        setPreferredSize(new Dimension(1200, 700));
        setFocusable(true);
        setLayout(new BorderLayout());
        errorMsg.setVisible(false);
        errorMsg.setForeground(Color.RED);
        errorMsg.setAlignmentX(CENTER_ALIGNMENT);

        try {
            bgImage = ImageIO.read(getClass().getResourceAsStream("/resources/images/islandBg.png"));
        } catch (Exception e) {
            System.err.println("Could not load background image!");
            e.printStackTrace();
        }

        createLoginForm();
        PageLayout.createMenuButton(this);
    }

    private void createLoginForm() {
        // wrapper for centering
        JPanel wrapper = new JPanel(new GridBagLayout());
        wrapper.setOpaque(false);

        // login form
        Flexbox form = new Flexbox(true);
        form.setPreferredSize(new Dimension(700, 500));
        form.setAlignmentX(CENTER_ALIGNMENT);
        form.setBackground(ColorManager.primarySand);
        form.addPadding(60);

        Header title = new Header("Welcome back!");
        title.setAlignmentX(CENTER_ALIGNMENT);
        title.setForeground(ColorManager.primaryBrown);

        // input box for username
        userField = new InputField(InputField.Type.BASIC, "Username:");
        userField.setAlignmentX(CENTER_ALIGNMENT);

        // input box for password
        passField = new InputField(InputField.Type.PASSWORD, "Password:");
        passField.setAlignmentX(CENTER_ALIGNMENT);

        // login button
        StyledButton loginBtn = new StyledButton("LOGIN", StyledButton.ButtonStyle.RECT);
        loginBtn.addActionListener(e -> handleLogin());
        loginBtn.setAlignmentX(CENTER_ALIGNMENT);

        form.add(Box.createVerticalStrut(30));
        form.add(title);
        form.add(Box.createVerticalGlue());
        form.add(userField);
        form.add(Box.createVerticalGlue());
        form.add(passField);
        form.add(Box.createVerticalGlue());
        form.add(loginBtn);
        form.add(Box.createVerticalStrut(5));
        form.add(errorMsg);
        form.add(Box.createVerticalGlue());

        wrapper.add(form);
        add(wrapper, BorderLayout.CENTER);
    }

    private void handleLogin() {
        username = userField.getText();
        password = passField.getText();
        boolean success = SaveData.getData().logIn(username, password);
        if (success) {
            ScreenEventBus.publish("Player Screen", new PlayerScreenPage());
            errorMsg.setVisible(false);
        } else {
            System.out.println("Invalid Login");
            errorMsg.setVisible(true);
        }
        userField.setText("");
        passField.setText("");
        revalidate();
        repaint();
    }

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
