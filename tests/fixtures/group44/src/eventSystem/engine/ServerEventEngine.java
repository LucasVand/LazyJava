package eventSystem.engine;

import eventSystem.events.Event;
import eventSystem.events.EventCoder;
import eventSystem.events.ResponseEvent;
import eventSystem.events.StandaloneEvent;
import eventSystem.networking.RawEvent;
import eventSystem.networking.server.Server;
import eventSystem.networking.server.TraceableRawEvent;
import java.io.IOException;
import java.util.concurrent.LinkedBlockingQueue;

abstract public class ServerEventEngine implements AutoCloseable {
    Server server;
    Thread thread;

    public ServerEventEngine(int port) {

        this.server = new Server(port);
    }

    public void start() {
        this.server.start();
        startServerThread();
        this.server.setOnClientDisconnect((id) -> {
            this.handleClientDisconnect(id);
        });
    }

    public boolean isRunning() {
        return this.server.isRunning();
    }

    public void waitUntilStarted() {
        while (!this.server.isRunning()) {
        }
    }

    void startServerThread() {

        LinkedBlockingQueue<TraceableRawEvent> queue = this.server.readQueue();
        thread = new Thread(() -> {
            try {
                while (true) {
                    TraceableRawEvent traceableRaw;
                    while ((traceableRaw = queue.take()) != null) {
                        Event event = EventCoder.decodeEvent(traceableRaw.event());
                        Event returnEvent = handleEvent(traceableRaw.clientId(), event);

                        if (returnEvent != null) {
                            // if the returned event is not standalone
                            if (!(returnEvent instanceof StandaloneEvent)) {
                                System.out.println(
                                        "Error: Response from a ResponseEvent is not a standaloneEvent, Nested Response Events are not allowed, unable to send the response, fix this");
                            } else {
                                // set the same event id for the callback event
                                returnEvent.setEventId(event.eventId());
                                sendClient(traceableRaw.clientId(), (StandaloneEvent) returnEvent);
                            }
                        } else if (event instanceof ResponseEvent) {
                            // if we got a callback event but didnt expect one
                            System.err.println(
                                    "Error: Found response event which expects a response from handle event but no response was supplied, this behaviour is not recommended, fix this");
                        }
                        // dash.addRequest();
                    }
                }
            } catch (InterruptedException e) {
            } catch (Exception e) {
                System.out.println("Server Engine Thead Failed");
                System.out.println(e);
                e.printStackTrace();
            }
        }, "ServerEngineThread");

        thread.start();
    }

    abstract public Event handleEvent(int clientId, Event event);

    public void handleClientDisconnect(int clientId) {

    };

    public void broadcast(StandaloneEvent event) {
        try {
            RawEvent raw = EventCoder.encodeEvent(event);
            server.broadcast(raw);
        } catch (IOException e) {
            System.out.println("Unable to encode Event");
            System.out.println(e);
        }
    }

    public void broadcastNot(int[] excludeIds, StandaloneEvent event) {
        try {
            RawEvent raw = EventCoder.encodeEvent(event);
            server.broadcastNot(excludeIds, raw);
        } catch (IOException e) {
            System.out.println("Unable to encode Event");
            System.out.println(e);
        }
    }

    public void broadcastNot(int excludeId, StandaloneEvent event) {
        try {
            RawEvent raw = EventCoder.encodeEvent(event);
            server.broadcastNot(new int[] { excludeId }, raw);
        } catch (IOException e) {
            System.out.println("Unable to encode Event");
            System.out.println(e);
        }
    }

    // TODO: we want the server to be able to send responseEvents too
    public void sendClient(int id, StandaloneEvent event) {
        try {
            RawEvent raw = EventCoder.encodeEvent(event);
            server.writeClient(id, raw);
        } catch (IOException e) {
            System.out.println("Unable to encode Event");
            System.out.println(e);
        }
    }

    public void close() {
        thread.interrupt();
        server.close();
    }
}
