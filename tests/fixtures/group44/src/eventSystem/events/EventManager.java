package eventSystem.events;

import java.util.HashMap;
import java.util.Iterator;
import java.util.LinkedList;
import java.util.Map;
import java.util.function.Consumer;

import javax.swing.SwingUtilities;

/**
 * A class for managing incoming events and distributing them into the
 * registered listeners and callbacks, used internally in
 * {@code ClientEventEngine}, this class should not have to interacted with use
 * {@code ClientEventEngine} instead
 * 
 * @author Lucas Vanderwielen
 */
public class EventManager {
    // hold all the normal callbacks
    HashMap<Class<? extends Event>, LinkedList<EventCallback>> callbacks;
    // holds all the callbacks for responseEvents sent that are waiting for thier
    // responses
    HashMap<String, Consumer<? extends Event>> returnCallbacks;
    // hold the id of the next listener id
    int nextId;

    public EventManager() {
        callbacks = new HashMap<>();
        returnCallbacks = new HashMap<>();
        nextId = 0;
    }

    // gets the next listener id
    int getId() {
        return nextId++;
    }

    // this is used to register a listeners that is used for a response events
    // response
    public <T extends Event> void registerEventCallback(String eventId, Consumer<T> callback) {
        returnCallbacks.put(eventId, callback);
    }

    // adds a listener
    public <T extends StandaloneEvent> int addListener(Class<T> callbackClass,
            Consumer<T> callback) {

        // gets the list of callbacks associated with that class
        LinkedList<EventCallback> list = callbacks.get(callbackClass);
        // gets the next id
        int id = getId();
        // if list null then create the list else add to it
        if (list == null) {
            list = new LinkedList<>();
            list.add(new EventCallback(false, id, callback));

            callbacks.put(callbackClass, list);
        } else {
            list.add(new EventCallback(false, id, callback));
        }
        return id;
    }

    // adds a once listener
    public <T extends StandaloneEvent> int addListener(Class<T> callbackClass,
            Consumer<T> callback, boolean once) {

        // gets the list
        LinkedList<EventCallback> list = callbacks.get(callbackClass);
        int id = getId();
        if (list == null) {
            list = new LinkedList<>();
            // adds the once event callback
            list.add(new EventCallback(once, id, callback));

            callbacks.put(callbackClass, list);
        } else {
            list.add(new EventCallback(once, id, callback));
        }
        return id;
    }

    // this is kind of expensive so idk how i feel about it
    public void removeListener(int id) {
        // maps over the registered listeners and removes the one with the matching id
        for (Map.Entry<Class<? extends Event>, LinkedList<EventCallback>> entry : callbacks.entrySet()) {
            entry.getValue().removeIf((event) -> {
                return event.id() == id;
            });
        }
    }

    public <T extends Event> void removeAllType(Class<T> eventClass) {
        callbacks.get(eventClass).clear();
    }

    public void removeAll() {
        this.callbacks.clear();
    }

    // called when an event occurs
    public void eventOccured(Event event) {
        // see if it is a reponse to a response event
        Consumer<? extends Event> response = returnCallbacks.get(event.id);
        // if it is a response then remove the listen and call it
        if (response != null) {
            returnCallbacks.remove(event.id);
            consume(response, event);
            return;
        }

        Class<? extends Event> callbackClass = event.getClass();
        LinkedList<EventCallback> list = callbacks.get(callbackClass);
        if (list == null) {
            return;
        }

        Iterator<EventCallback> iter = list.iterator();

        while (iter.hasNext()) {
            EventCallback consumer = iter.next();
            if (consumer.once()) {
                iter.remove();
            }

            consume(consumer.callback(), event);
        }
    }

    @SuppressWarnings("unchecked")
    private <T extends Event> void consume(Consumer<? extends Event> callback, Event event) {
        Consumer<T> func = ((Consumer<T>) callback);

        // if on the swing event thread then we can run it, this should event happen but
        // just making sure
        if (SwingUtilities.isEventDispatchThread()) {
            func.accept((T) event);
        } else {
            // all events should run on the main swing event thread
            SwingUtilities.invokeLater(() -> {
                func.accept((T) event);
            });
        }
    }
}
