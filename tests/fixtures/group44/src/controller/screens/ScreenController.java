package controller.screens;

import java.awt.CardLayout;

import javax.swing.JFrame;
import javax.swing.JPanel;

import controller.Controller;
import controller.screens.screenEvent.ScreenEvent;
import controller.screens.screenEvent.ScreenEventBus;
import controller.screens.screenEvent.ScreenEventListener;
import gameplayScreens.MainGame;
import playerScreen.FindRoomsPage;

/**
 * ScreenController controls which screens are shown on the JFrame at any given time.
 * It uses a cardLayout to register and show different screens based on screen name.
 * This class implements two interfaces: Controller and ScreenEventListener.
 * <p>
 * As a ScreenEventListener, this class listens for ScreenEvent's provided by the
 * ScreenEventBus.
 * <p>
 * As a Controller, this class will be able to register and show screens
 * 
 * @author Sam Deitz
 * @see controller.screens.screenEvent.ScreenEventBus
 * @see controller.screens.screenEvent.ScreenEvent
 * @see controller.screens.screenEvent.ScreenEventListener
 * @see controller.Controller
 */
public class ScreenController implements ScreenEventListener, Controller {

    /**
     * Main Window
     */
    private final JFrame window;

    /**
     * Panel housing pages
     */
    private final JPanel cardContainer;

    /**
     * Card layout to manage which page is shown
     */
    private final CardLayout cardLayout;

    /**
     * Dynamic screen to be removed on change of screen
     */
    private JPanel toBeRemoved = null;


    /**
     * Initialize a new game window
     */
    public ScreenController(JFrame window) {
        this.window = window;
        this.cardLayout = new CardLayout();
        this.cardContainer = new JPanel(cardLayout);

        // The JFrame only ever holds this one container
        this.window.add(cardContainer);
        ScreenEventBus.subscribe(this);
    }

    /**
     * Register a screen into the cardlayout
     * 
     * @param screenName
     * @param screenPanel
     */
    @Override
    public void registerScreen(String screenName, JPanel screenPanel) {
        cardContainer.add(screenPanel, screenName);
    }

    /**
     * Show the window
     */
    public void start() {
        window.pack();
        window.setLocationRelativeTo(null);
        window.setVisible(true);
    }

    /**
     * Change the visible screen to the given screen
     * 
     * @param screenName string name of the screen
     */
    @Override
    public void showScreen(String screenName) {
        // change screen in frame
        cardLayout.show(cardContainer, screenName);

        // clean last screen
        cleanLast();

        // focus key listeners
        cardContainer.requestFocusInWindow();
    }

    /**
     * Clean dynamic components
     */
    private void cleanLast() {

        // if there is a dynamic screen to be removed
        if (toBeRemoved != null) {

            // if it is the find rooms page close the room finding server
            if (toBeRemoved instanceof FindRoomsPage) {
                ((FindRoomsPage)toBeRemoved).getRoomFinder().stopSearching();
            }

            // if it is a gameplay page
            if (toBeRemoved instanceof MainGame) {

                // if they are in the waiting room and they are the host close the broadcast server
                if (((MainGame)toBeRemoved).getController().getRoomServer() != null) ((MainGame)toBeRemoved).getController().getRoomServer().stopBroadcasting();
                
                // close the game server
                ((MainGame)toBeRemoved).getState().close();
            }

            // remove the screen from the cardlayout
            cardContainer.remove(toBeRemoved);

            // set the screen to be removed back to null
            toBeRemoved = null;
        }
    }


    @Override
    public void requestScreenChange(ScreenEvent event) {

        // switch for page direction based on event enum
        switch (event) {
            case GO_TO_ADMIN_CONTROLS -> showScreen("ADMIN_CONTROLS");

            case GO_TO_CREATE_ACCOUNT -> showScreen("CREATE_ACCOUNT");

            case GO_TO_ADMIN_LOGIN -> showScreen("ADMIN_LOGIN");

            case GO_TO_INSTRUCTIONS -> showScreen("INSTRUCTIONS");

            case GO_TO_HIGH_SCORES -> showScreen("HIGH_SCORES");

            case GO_TO_LOGIN -> showScreen("LOGIN");

            case GO_TO_MAIN_MENU -> showScreen("MAIN_MENU");

            case JOIN_ROOM -> showScreen("WAITING_ROOM");

            case GO_TO_PLAYER_SCREEN -> showScreen("PLAYER_SCREEN");

            default -> {
            }

        }
    }

    
    @Override
    public void requestDynamicScreen(String screenName, JPanel screenPanel) {

        // register and show dynamic screen
        registerScreen(screenName, screenPanel);
        showScreen(screenName);

        // add this component to be removed
        toBeRemoved = screenPanel;
    }
}
