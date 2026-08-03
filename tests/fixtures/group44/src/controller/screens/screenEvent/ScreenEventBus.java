package controller.screens.screenEvent;

import java.util.ArrayList;

import javax.swing.JPanel;

/**
 * The ScreenEventBus class bridges the communication between a screen controller and
 * programmer in a UI class. It will post ScreenEvents to all listeners for execution.
 * <p>
 * This class allows abstraction between the UI components and ScreenController, acting
 * as the middleman between the two.
 * 
 * @author Sam Deitz
 * @see controller.screens.screenEvent.ScreenEvent
 * @see controller.screens.ScreenController
 */
public class ScreenEventBus {
    
    /**
     * All listeners that will execute based on events
     */
    private final static ArrayList<ScreenEventListener> listeners = new ArrayList<>();

    /**
     * Subscribe a listener to screen events
     * 
     * @param listener listener
     */
    public static void subscribe(ScreenEventListener listener) {
        listeners.add(listener);
    }

    /**
     * Propogate an event to the listeners
     * 
     * {@code btn.addActionListener(e -> controller.publish(ScreenEvent.GO_TO_MAIN_PAGE))}
     * 
     * @param event screen change event
     */
    public static void publish(ScreenEvent event) {
        for (ScreenEventListener listener : listeners) {
            listener.requestScreenChange(event);
        }
    }

    /**
     * publish a dynamic screen
     * 
     * {@code btn.addActionListener(e -> controller.publish(ScreenEvent.GO_TO_MAIN_PAGE))}
     * 
     * @param screenName string representing screen
     * @param screenPanel JPanel with screen UI
     */
    public static void publish(String screenName, JPanel screenPanel) {
        for (ScreenEventListener listener : listeners) {
            listener.requestDynamicScreen(screenName, screenPanel);
        }
    }


}
