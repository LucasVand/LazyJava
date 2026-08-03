package controller.screens.screenEvent;

import javax.swing.JPanel;

/**
 * The ScreenEventListener interface is for classes that will listen for screen event changes.
 * It must implement two methods:
 * <p>
 * requestScreenChange: posts a screen event to change the visible screen
 * <p>
 * RequestDynamiScreen: posts a dynamic screen change to be initialized and shown
 * 
 * @author Sam Deitz
 */
public interface ScreenEventListener {

    /**
     * Request a change of screen
     * @param event new screen
     */
    void requestScreenChange(ScreenEvent event);

    /**
     * Request a dynamic screen to be added and shown
     * @param screenName name of screen
     * @param screenPanel panel corrosponding to name
     */
    void requestDynamicScreen(String screenName, JPanel screenPanel);
}