package eventSystem.engine;

import java.io.IOException;
import java.util.concurrent.LinkedBlockingQueue;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.BiConsumer;
import java.util.function.Consumer;

import eventSystem.events.Event;
import eventSystem.events.EventCoder;
import eventSystem.events.EventManager;
import eventSystem.events.ResponseEvent;
import eventSystem.events.StandaloneEvent;
import eventSystem.networking.RawEvent;
import eventSystem.networking.client.Client;

/**
 * This class is the main way that our application interacts with the server. It
 * handles all the networking requests along with
 * all of the event loop functions
 * <p>
 *
 * *Usage*
 *
 * {@snippet :
 * ClientEventEngine engine = new ClientEventEngine("10.0.0.172", 5001, (self, e) -> {
 *
 * });
 * }
 * The Ipv4 address of the computer running the server must be given,
 * the port that the server is bound to, along with a closure that runs when all
 * initalization is complete or an error occurs during init and the engine is
 * ready to send a recive events,
 * this closure takes in an error that might of occured during init please
 * handle
 *
 * *Initalization*
 * This class may take time to initalize and connect to the server, during this
 * time events will not be sent, events will be cached and sent as
 * early as possible (i.e when we get a connection to the server),
 * {@code isInialized()} can be used to see whether it is
 * initalized, along with the initalization closure that it passed into the
 * constructor
 * <p>
 * This class should be created once when the player joins a room. After that
 * its referance should be passed around.
 * <p>
 * *Clean Up*
 * This objects lifetime will persist until {@code close()} is called, this
 * function will terminate all threads associated to it. If {@code close()}
 * is not called this object will persist even when it goes out of scope as it
 * has background threads that still are running and have access to it
 * 
 * @author Lucas Vanderwielen
 */
public class ClientEventEngine implements AutoCloseable {
    Client client;
    EventManager eventManager;
    Thread thread;

    /**
     * Main constructor for ClientEventEngine, should be called when the player is
     * trying to join a room
     * {@snippet :
     * ClientEventEngine engine = new ClientEventEngine("10.0.0.172", 5001, (self, e) -> {
     *
     * });
     * }
     * The Ipv4 address of the computer running the server must be given along with
     * the port that the server is bound to along with a closure that runs when all
     * initalization is complete or an error occurs and the engine is ready to send
     * a recive events, this closure takes in an error that might of occured during
     * init please handle and a referance to the object itself
     *
     * @param addr         the IPv4 address of the server as a string
     *                     {@code "172.168.192.13"}
     * @param port         the port of the server to connect
     *
     * @param initFunction this functions runs when all inialization is completed,
     *                     this has a param of type {@code Exception}, this holds an
     *                     exceptions that occured during during init and
     *                     {@code ClientEventEngine} which is reference to itself
     */
    public ClientEventEngine(String addr, int port, BiConsumer<ClientEventEngine, IOException> initFunction) {
        innerConstructor(addr, port, initFunction);
    }

    /**
     * Main constructor for ClientEventEngine, should be called when the player is
     * trying to join a room
     * {@snippet :
     * ClientEventEngine engine = new ClientEventEngine("10.0.0.172", 5001);
     * }
     * The Ipv4 address of the computer running the server must be given along with
     * the port that the server is bound
     *
     * @param addr the IPv4 address of the server as a string
     *             {@code "172.168.192.13"}
     * @param port the port of the server to connect
     *
     */
    public ClientEventEngine(String addr, int port) {
        innerConstructor(addr, port, (engine, exception) -> {
        });
    }

    // this is here because java does not have optional values
    void innerConstructor(String addr, int port, BiConsumer<ClientEventEngine, IOException> initFunction) {
        Consumer<IOException> clientCallback = (ex) -> {
            initFunction.accept(this, ex);
        };

        this.eventManager = new EventManager();

        this.client = new Client(addr, port, clientCallback);

        thread = new Thread(() -> {
            eventLoop();
        }, "ClientEngineEventThread");
        thread.start();
    }

    public void setOnDisconnect(Runnable callback) {
        this.client.setOnDisconnect(callback);
    }


    // this is where the event loop lives
    void eventLoop() {
        // we gain a referance to the client read queue
        LinkedBlockingQueue<RawEvent> queue = client.readQueue();
        try {
            // this loop runs forever until the thread is killed
            while (true) {
                RawEvent raw;
                // get the raw event from the client object, this is blocking and will wait
                // until there is an event to take
                if ((raw = queue.take()) != null) {
                    // put the event through the decoder
                    Event event = EventCoder.decodeEvent(raw);
                    // call the event manager to process that an event happened
                    eventManager.eventOccured(event);
                }
            }
        } catch (InterruptedException e) {
            // i dont care about interrupt exception, im trying to kill the thread it should
            // die gracefully
        } catch (Exception e) {
            // TODO: add better error handling, this is running user code so we need to make
            // sure to handle everything
            System.out.println("Client Event Thread Failed");
            e.printStackTrace();
        }
    }

    /**
     * Sends a {@code Standalone } event to the server, this function is non
     * blocking and returns immediatly
     *
     * @param event this is the event that will be sent to the server, must extend
     *              {@code StandaloneEvent}.
     *
     *              <p>
     *              </p>
     * @see {@code sendEvent(ResponseEvent, Consumer<StandaloneEvent>)} for sending
     *      a {@code ResponseEvent }
     *
     */
    public void sendEvent(StandaloneEvent event) {

        sendEvent((Event) event);
    }

    // this is the underlying function that actually sends the events to the client
    // to be sent to the server
    void sendEvent(Event event) {
        try {
            // encode the event into a raw event
            RawEvent raw = EventCoder.encodeEvent(event);
            // call write on the client to write this raw event to the server
            client.write(raw);
        } catch (IOException e) {
            // TODO: better error handling here
            System.out.println("Could not send event");
            System.out.println(e);
        }
    }

    /**
     * Sends a {@code ResponseEvent } event to the server with the expectation of a
     * response, this function is non blocking and returns immediatly. When this
     * event is sent the server will process it then sent a response of the
     * specified type
     *
     * @param <T>      this is the type of the {@code ResponseEvent}
     * @param <K>      this is the response type from the {@code ResponseEvent}, the
     *                 callback param will be of this type, must extend
     *                 {@code StandaloneEvent}
     *
     * @param event    this is the event that will be sent to the server, must
     *                 extend {@code ResponseEvent}.
     * @param callback this is the callback that will be executed when the server
     *                 receives and handles the event, the param type is determined
     *                 buy the {@code ResponseEvent} object
     *
     *                 <p>
     *                 </p>
     * @see {@code sendEvent(StandaloneEvent)} for sending
     *      a {@code StandaloneEvent }
     *
     */
    public <K extends StandaloneEvent, T extends ResponseEvent<K>> void sendEvent(T event,
            Consumer<K> callback) {
        eventManager.registerEventCallback(event.eventId(), callback);
        sendEvent(event);
    }

    /**
     * Sets an optional one time listener for a specific event, when the server
     * sends an event of type {@code callbackClass}, the closure supplied will be
     * run and if {@code once} is set then the closure will be discarded
     *
     * @param <T>           this is the type of event that will be listened for,
     *                      must extend {@code StandaloneEvent}
     *
     * @param callbackClass this is the class of the event that the listener will
     *                      listen for extend {@code StandaloneEvent}.
     * @param callback      this is the callback that will be executed when the
     *                      server sends an event of this type, the param will be of
     *                      type {@code <T>}
     * @param once          a toogle for if this is a persistant listener or it will
     *                      only listen for a single event then be discarded
     *
     * @return the id of the listener, this id can later be
     *         used to remove the
     *         listener
     *
     *         <p>
     *         </p>
     * @see {@code addListener(Class<T>, Consumer<T>)} for persistant listener
     *
     */
    public <T extends StandaloneEvent> int addListener(Class<T> callbackClass,
            Consumer<T> callback, boolean once) {
        return eventManager.addListener(callbackClass, callback, once);
    }

    /**
     * Sets an listener for a specific event, when the server
     * sends an event of type {@code callbackClass}, the closure supplied will be
     * run, this is persistant so the closure might run many times if the server
     * sends many events
     *
     * @param <T>           this is the type of event that will be listened for,
     *                      must extend {@code StandaloneEvent}
     *
     * @param callbackClass this is the class of the event that the listener will
     *                      listen for extend {@code StandaloneEvent}.
     * @param callback      this is the callback that will be executed when the
     *                      server sends an event of this type, the param will be of
     *                      type {@code <T>}
     *
     * @return the id of the listener, this id can later be used to remove the
     *         listener
     *
     *         <p>
     *         </p>
     * @see {@code addListener(Class<T>, Consumer<T>, boolean)} for a non persistant
     *      listener
     *
     */
    public <T extends StandaloneEvent> int addListener(Class<T> callbackClass,
            Consumer<T> callback) {
        return eventManager.addListener(callbackClass, callback);
    }

    /**
     * Removes all current listeners, all listeners and closure will be discarded,
     * current waiting responses from {@code ResponseEvent} will not be discarded
     * and those will still function as normal
     * 
     * <p>
     * </p>
     * 
     * @see {@code removeAllType(Class<T>)} to remove all listeners of a class
     * @see {@code removeListener(int)} to remove a specific listener
     *
     * 
     */
    public void removeAll() {
        eventManager.removeAll();
    }

    /**
     * Removes all listeners of a specific class, current waiting responses from
     * {@code ResponseEvent} will not be discarded and those will still function as
     * normal
     *
     * 
     * @param <T>           the type of event where the listeners will be removed,
     *                      must extend {@code Event}
     *
     * @param callbackClass this is the type of the class for which the listeners
     *                      with that type will be removed, must extend
     *                      {@code Event}
     *
     *                      <p>
     *                      </p>
     * @see {@code removeAll()} to remove all listeners
     * @see {@code removeListener(int)} to remove a specific listener
     */
    public <T extends Event> void removeAllType(Class<T> callbackClass) {
        eventManager.removeAllType(callbackClass);
    }

    /**
     * Removes a specific listener with the id supplied
     *
     * @param callbackId the id of the listener to remove
     *
     *                   <p>
     *                   </p>
     * @see {@code removeAllType(Class<T>)} to remove all listeners of a class
     * @see {@code removeAll()} to remove all listeners
     */
    public void removeListener(int callbackId) {
        eventManager.removeListener(callbackId);
    }

    /**
     * Disconnects the client from this server and closes all connections, this
     * function must be called when
     * you want to get rid of this object, if this is not called the object will
     * never be collected by the GC because it has background threads that remain
     * with referances to it
     */
    public void close() {
        client.disconnect();
        this.thread.interrupt();
    }

    /**
     * Checks whether this engine has been initalized and connected and it ready to
     * send and recive events, if the server is closed then this is set back to
     * false events that are sent before this will be cached and sent at
     * the next available time
     *
     * @return whether this engine is initalized and is connected
     */

    public AtomicBoolean isConnected() {
        return client.getConnected();
    }

    /**
     * Waits until the engine is ready and connected, this function is blocking and
     * will wait until {@code isConnected()} is true, this is optional and the
     * client can safely send events right after initalization
     */
    public void waitUntilConnected() {
        while (!this.client.getConnected().get()) {
        }
    }
}
