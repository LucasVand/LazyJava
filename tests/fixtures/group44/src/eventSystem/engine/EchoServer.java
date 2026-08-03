package eventSystem.engine;

import eventSystem.events.Event;
import eventSystem.events.StandaloneEvent;

/**
 * EchoServer, a server which just echos back all events
 *
 * @author Lucas Vanderwielen
 */
public class EchoServer extends ServerEventEngine {

    public EchoServer(int port) {
        super(port);
    }

    @Override
    public Event handleEvent(int clientId, Event event) {
        broadcast((StandaloneEvent) event);
        return null;
    }
}
