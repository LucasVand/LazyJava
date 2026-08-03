package eventSystem.networking.server;

import java.awt.Checkbox;
import java.io.IOException;
import java.net.InetSocketAddress;
import java.nio.ByteBuffer;
import java.nio.channels.ClosedSelectorException;
import java.nio.channels.SelectableChannel;
import java.nio.channels.SelectionKey;
import java.nio.channels.Selector;
import java.nio.channels.ServerSocketChannel;
import java.nio.channels.SocketChannel;
import java.util.ArrayDeque;
import java.util.HashMap;
import java.util.Iterator;
import java.util.Map;
import java.util.Queue;
import java.util.concurrent.LinkedBlockingQueue;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.Consumer;

import eventSystem.networking.RawEvent;

/**
 * The main class responsible the server, this is what reads incoming events,
 * connects to clients and handles events
 * 
 * @author Lucas Vanderwielen
 */
public class Server implements AutoCloseable {
    // this is where all read events are put
    LinkedBlockingQueue<TraceableRawEvent> toReadQueue;
    // this is the manager of all the client connections
    Selector selector;
    // this is a list of all the connected clients
    // key is the client id
    HashMap<Integer, ClientState> clients;
    // this is the thread that reads from the selector
    Thread networkThread;
    // port
    int port;
    // a counter for assigning client ids
    int clientIdCounter;

    AtomicBoolean running;

    Consumer<Integer> onClientDisconnect;

    // every attached client has an associated state
    public static class ClientState {

        // the read buffer, holds data before it can be processsed
        public ByteBuffer readBuffer = ByteBuffer.allocate(RawEvent.MAX_BYTEBUFFER);
        // this is a queue of bufs the we want to write
        public Queue<ByteBuffer> writeQueue = new ArrayDeque<>();

        // the sockets
        SocketChannel socket;
        // the key, this is very important
        SelectionKey key;
        public int id;

        public ClientState(int id, SocketChannel socket, SelectionKey key) {
            this.socket = socket;
            this.key = key;
            this.id = id;
        }
    }

    // gets the next client id
    int nextId() {
        int id = clientIdCounter;
        clientIdCounter++;
        return id;
    }

    public Server(int port) {
        this.toReadQueue = new LinkedBlockingQueue<>();
        this.port = port;
        this.clients = new HashMap<>();
        this.clientIdCounter = 0;
        this.running = new AtomicBoolean();
    }

    public void setOnClientDisconnect(Consumer<Integer> callback) {
        this.onClientDisconnect = callback;
    }

    public void start() {
        serverLoop(this.port);
    }

    public LinkedBlockingQueue<TraceableRawEvent> readQueue() {
        return this.toReadQueue;
    }

    public HashMap<Integer, ClientState> getClients() {
        return this.clients;
    }

    void serverLoop(int port) {
        networkThread = new Thread(() -> {
            try {
                // create the selector
                selector = Selector.open();

                // creates a server, this is what allows clients to connect
                // when connecting they connect to this server and the sever then passes on the
                // socket for us to handle
                ServerSocketChannel server = ServerSocketChannel.open();
                server.bind(new InetSocketAddress("0.0.0.0", port));
                server.configureBlocking(false);

                // register the server with the selector and tells the selector to listen for
                // clients trying to accept
                server.register(selector, SelectionKey.OP_ACCEPT);

                startServerLoop();
            } catch (InterruptedException e) {
            } catch (ClosedSelectorException e) {
            } catch (IOException e) {
                System.out.println("Network Thread Threw");
                System.out.println(e);
                //
                // we should put a server crash event here so that outside threads know the
                // server has died
                // enqueue a special raw event here
            }
        }, "Server Network Thread");
        networkThread.start();

    }

    public boolean isRunning() {
        return this.running.get();
    }

    void startServerLoop() throws IOException, InterruptedException {
        while (true) {
            this.running.setRelease(true);
            // this blocks until the selector has something
            selector.select();

            Iterator<SelectionKey> iter = selector.selectedKeys().iterator();
            while (iter.hasNext()) {
                SelectionKey key = iter.next();
                iter.remove();

                // this is mainly to catch disconnects but needs improvement
                try {
                    // make sure the key is valid
                    if (!key.isValid())
                        continue;

                    // if we have a new connection waiting on the ServerSocketChannel key
                    if (key.isAcceptable()) {
                        this.acceptNewConnection(selector, key);
                        continue;
                    }

                    // if we can read
                    if (key.isReadable())
                        this.read(key);

                    // if we can write
                    if (key.isWritable())
                        this.write(key);
                } catch (Exception e) {
                    resolveCancelledKey(key);
                }
            }
        }

    }

    void acceptNewConnection(Selector sel, SelectionKey key) throws IOException {
        // this is the servers key
        ServerSocketChannel server = (ServerSocketChannel) key.channel();

        // accept the new connection
        SocketChannel client = server.accept();

        if (client == null)
            return;

        // configure blocking
        client.configureBlocking(false);
        // get the next id and create client state
        int id = nextId();
        // create a new client state, setting the key to null because it does not exist
        // yet
        ClientState clientState = new ClientState(id, client, null);

        // register the client with the selector
        SelectionKey newKey = client.register(sel, SelectionKey.OP_READ, clientState);
        // set the client key, the key now exists and we can set it
        clientState.key = newKey;

        // add to clients list
        clients.put(id, clientState);

    }

    void read(SelectionKey key) throws IOException {
        // get the socket and state
        SocketChannel client = (SocketChannel) key.channel();
        ClientState state = (ClientState) key.attachment();

        // read the bytes from the socket into the state read buffer
        int bytesRead = client.read(state.readBuffer);

        // if -1 bytes disconnect the client, that means disconnect for some reason
        if (bytesRead == -1) {
            resolveCancelledKey(key);
            return;
        }

        ByteBuffer buf = state.readBuffer;
        buf.flip();
        this.parseMessage(buf, state.id);
        buf.compact();
    }

    void write(SelectionKey key) throws IOException {

        // the socket and state
        SocketChannel channel = (SocketChannel) key.channel();
        ClientState state = (ClientState) key.attachment();

        // while there are still buffers to be written
        while (!state.writeQueue.isEmpty()) {
            // get the buffer but do not dequeue
            // this is because we might not be able to dump the whole buffer
            // into the socket
            ByteBuffer buf = state.writeQueue.peek();

            // write the buf to the socket
            channel.write(buf);

            if (buf.hasRemaining()) {
                // socket is full we need to wait to write more because
                // the channel is full
                break;
            }

            // if we make it down here then we have dumped the whole buffer
            state.writeQueue.poll();
        }

        // if we have no buffers left remove the write interest
        if (state.writeQueue.isEmpty()) {
            // remove the OP_WRITE
            key.interestOps(key.interestOps() & ~SelectionKey.OP_WRITE);
        }
    }

    void parseMessage(ByteBuffer buf, int clientId) throws IOException {
        // [4 byte content length][1 byte event type][content]
        RawEvent event = RawEvent.decode(buf);

        if (event != null) {
            // add the parsed event to the queue
            boolean res = toReadQueue.offer(new TraceableRawEvent(clientId, event));
            if (!res) {
                throw new RuntimeException("Unable to enqueue event");
            }
            // if there is still more to read then parse the message again
            if (buf.hasRemaining()) {
                parseMessage(buf, clientId);
            }
        }

    }

    void ensureKeyValidity(SelectionKey key) {
        if (key.isValid()) {
            return;
        }

        try {
            resolveCancelledKey(key);
        } catch (IOException e) {
            System.out.println("failed to resolve canceled key");
        }
    }

    void resolveCancelledKey(SelectionKey key) throws IOException {

        System.out.println("Resolving Cancelled Key");

        // close socket
        SocketChannel socket = (SocketChannel) key.channel();
        ClientState state = (ClientState) key.attachment();

        key.cancel();

        // remove from clients list
        if (clients.containsKey(state.id)) {
            this.onClientDisconnect.accept(state.id);
        }
        clients.remove(state.id);

        if (socket != null) {
            socket.close();
        }
    }

    public void removeClient(int id) throws IOException {
        SelectionKey key = clients.get(id).key;
        resolveCancelledKey(key);
    }

    public void broadcast(RawEvent event) {
        broadcastNot(new int[] {}, event);
    }

    public void broadcastNot(int[] excludeIds, RawEvent event) {
        if (event == null) {
            throw new RuntimeException("Cannot broadcast null");
        }

        // loop over all the clients
        for (Map.Entry<Integer, ClientState> entry : clients.entrySet()) {
            // if the key is included in the exclude ids
            int id = entry.getKey();
            boolean isExcluded = false;
            for (int excludeId : excludeIds) {
                if (excludeId == id)
                    isExcluded = true;
            }

            // if its not excluded then write the event to the client
            if (!isExcluded) {
                ClientState state = entry.getValue();
                writeClientState(state, event);
            }
        }
    }

    public void writeClient(int clientId, RawEvent event) {
        ClientState state = clients.get(clientId);

        if (state != null) {
            writeClientState(state, event);
        } else {
            throw new RuntimeException("Client with id does not exist, ID: " + clientId);
        }
    }

    void writeClientState(ClientState state, RawEvent event) {
        ensureKeyValidity(state.key);

        // encode the event into a buf
        ByteBuffer buf = RawEvent.encode(event);
        // add the buffer the clients write queue
        state.writeQueue.add(buf);
        // tell the selector that we want to write
        state.key.interestOps(state.key.interestOps() | SelectionKey.OP_WRITE);
        // wake the selector up
        state.key.selector().wakeup();
    }

    public void close() {
        try {
            onClientDisconnect = (e) -> {
            };

            // Iterate over all registered keys
            Iterator<SelectionKey> keys = this.selector.keys().iterator();
            while (keys.hasNext()) {
                SelectionKey key = keys.next();
                SelectableChannel channel = key.channel();
                channel.close();
                key.cancel();
            }
            // Finally, close the selector itself
            networkThread.interrupt();
            selector.close();
        } catch (ClosedSelectorException e) {
        } catch (Exception e) {
            System.out.println("Unable to close Server, Error: " + e);
        }
        System.out.println("Server Closed sucessfully");
    }
}
