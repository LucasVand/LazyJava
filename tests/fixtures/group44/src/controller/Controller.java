package controller;

import javax.swing.JPanel;


/**
 * The Controller interface is for classes that will control screen flow.
 * It contains two manditory methods:
 * <p>
 * registerScreen: allows registering screens to the flow
 * <p>
 * showScreen: allows for showing screens to the user
 * 
 * @author Sam Deitz
 */
public interface Controller {

    /**
     * Register a screen to the controller
     * @param screenName name of the screen
     * @param screenPanel panel with screen UI
     */
    public void registerScreen(String screenName, JPanel screenPanel);

    /**
     * Show the screen on the UI window
     * @param screenName name of the screen to be shown
     */
    public void showScreen(String screenName);
}
