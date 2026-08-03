package UIComponents;
import java.awt.BorderLayout;
import java.awt.Component;
import java.awt.event.ActionListener;

import javax.swing.Box;
import javax.swing.JPanel;

import controller.screens.screenEvent.ScreenEvent;
import controller.screens.screenEvent.ScreenEventBus;

/**
 * Class holding static methods for layout creation
 * 
 * @author Sam Deitz
 */
public class PageLayout {

    /**
     * Create a Main Menu button in the bottom right corner of a JPanel
     * with border layout
     * 
     * @param target target panel
     */
    public static void createMenuButton(JPanel target) {              
        StyledButton btn = new StyledButton("Main Menu", StyledButton.ButtonStyle.ROUNDED_RECT);
        btn.addActionListener(e -> ScreenEventBus.publish(ScreenEvent.GO_TO_MAIN_MENU));

        initializeMenu(target, btn);
    }

    /**
     * Create a back button in the bottom right corner of a JPanel
     * with border layout. Also takes in an ActionListener to be used
     * for the button.
     * 
     * {@snippet : 
     * PageLayout.createBackButton(myPanel, e -> {
     *     System.out.println("Clicked");
     * });
     * }
     * 
     * @param target target panel
     * @param callback action listener to be activated on click of the button
     */
    public static void createBackButton(JPanel target, ActionListener callback) {
        StyledButton btn = new StyledButton("Back", StyledButton.ButtonStyle.RECT);
        btn.addActionListener(callback);

        initializeMenu(target, btn);
    }

    /**
     * Helper method to initialize the menu for the create button methods
     * @param target target panel
     * @param b button
     */
    private static void initializeMenu(JPanel target, StyledButton b) {
        // Set up the menu
        Flexbox btnMenu = new Flexbox();
        btnMenu.setOpaque(false);
        btnMenu.addPadding(20);
        b.setAlignmentX(Component.LEFT_ALIGNMENT);

        // add the components
        btnMenu.add(Box.createHorizontalGlue());
        btnMenu.add(b);
        target.add(btnMenu, BorderLayout.SOUTH);
    }
}
