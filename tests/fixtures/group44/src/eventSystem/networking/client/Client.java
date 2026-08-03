package eventSystem.networking.client;

import java.io.IOException;
import java.net.InetSocketAddress;
import java.nio.ByteBuffer;
import java.nio.channels.CancelledKeyException;
import java.nio.channels.ClosedSelectorException;
import java.nio.channels.SelectionKey;
import java.nio.channels.Selector;
import java.nio.channels.SocketChannel;
import java.util.ArrayDeque;
import java.util.Iterator;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.concurrent.LinkedBlockingQueue;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.Consumer;

import eventSystem.networking.RawEvent;

/**
 * The main class responsible for connecting, sending and recieving data from
 * the server, this class should not be directly interacted with
 * {@code ClientEventEngine} is a wrapper around it and {@code EventManager}
 * which provides easy to use functions and operations for communicating with
 * the server
 * 
 * @author Lucas Vanderwielen
 */
public class Client {
    Thread thread;
    // the queue the event manager reads from
    LinkedBlockingQueue<RawEvent> toReadQueue;

    // a queue of buffers, when writing an event, a buffer is added to this queue
    Queue<ByteBuffer> writeQueue;
    // the intermediate buffer where data is read from the socket before it is
    // turned into a rawEvent and pushed to the readQueue
    ByteBuffer readBuf;

    // address and port of the socket
    String addr;
    int port;

    // this is the actual socket that is connected to the server
    SocketChannel socket;
    // the key registered with the selector
    // very important
    SelectionKey key;

    Selector selector;

    // this is true after all initalization has been completed and it is safe to
    // start writing and reading
    AtomicBoolean isConnected;

    // this is a queue for start up when not initalized we send written events here
    // then dump this when we first can
    ConcurrentLinkedQueue<RawEvent> startUpQueue;

    // called when disconnected
    Runnable onDisconnect;

    public Client(String addr, int port, Consumer<IOException> init) {
        this.startUpQueue = new ConcurrentLinkedQueue<>();
        this.toReadQueue = new LinkedBlockingQueue<>();

        this.readBuf = ByteBuffer.allocate(RawEvent.MAX_BYTEBUFFER);
        this.writeQueue = new ArrayDeque<>();
        this.addr = addr;
        this.port = port;

        this.isConnected = new AtomicBoolean();
        this.onDisconnect = () -> {
        };

        this.thread = new Thread(() -> {
            clientLoop(init);
        }, "Client Networking Thread");
        this.thread.start();

    }

    public void setOnDisconnect(Runnable callback) {
        this.onDisconnect = callback;
    }

    void clientLoop(Consumer<IOException> init) {
        try {
            // create the socket that the client lives on
            this.socket = SocketChannel.open(new InetSocketAddress(addr, port));
            // create the selector
            // this manages when to wake up and read/write data to the socket
            selector = Selector.open();

            // set blocking false to write and read ops are non blocking
            this.socket.configureBlocking(false);

            // register our socket with the selector and tell it we want to be able to read
            // from it
            this.key = socket.register(selector, SelectionKey.OP_READ);

            // once all initalization is completed then run this function with null to show
            // no errors
            init.accept(null);
            this.isConnected.setRelease(true);

            flushStartQueue();
            selectorLoop();

        } catch (InterruptedException e) {

        } catch (ClosedSelectorException e) {
        } catch (IOException e) {
            // TODO: add better error handling

            System.out.println("Connection Failed, IP : " + addr + " Port : " + port);
            System.out.println(e);
            if (!isConnected.get()) {
                init.accept(e);
            }
        } finally {
            invalidateKey();
        }
    }

    // flushes any events in the start queue and sends them
    void flushStartQueue() {
        // flush the queue
        RawEvent ev;
        while ((ev = startUpQueue.poll()) != null) {
            write(ev);
        }
    }

    void selectorLoop() throws IOException, InterruptedException {
        while (true) {
            // Blocks here until an event occurs
            // the selector knows when there is data to read and it wakes up from here
            selector.select();

            // Get the keys (events) that happened
            Iterator<SelectionKey> iter = selector.selectedKeys().iterator();

            while (iter.hasNext()) {
                SelectionKey key = iter.next();
                iter.remove();

                try {
                    if (!key.isValid()) {
                        continue;
                    }

                    if (key.isReadable()) {
                        socketRead(key);
                    }

                    if (key.isWritable()) {
                        socketWrite(key);
                    }
                } catch (CancelledKeyException e) {
                    invalidateKey();
                    System.out.println("Closed Client Connection");
                }
            }
        }

    }

    // copied from server
    void socketWrite(SelectionKey key) throws IOException {
        // the socket
        SocketChannel channel = (SocketChannel) key.channel();

        // while there are still buffers to be written
        while (!writeQueue.isEmpty()) {
            // get the buffer but do not dequeue
            // this is because we might not be able to dump the whole buffer
            // into the socket
            ByteBuffer buf = writeQueue.peek();

            // write the buf to the socket
            channel.write(buf);

            if (buf.hasRemaining()) {
                // socket is full we need to wait to write more because
                // the channel is full
                break;
            }

            // if we make it down here then we have dumped the whole buffer
            writeQueue.poll();
        }

        // if we have no buffers left remove the write interest
        if (writeQueue.isEmpty()) {
            // remove the OP_WRITE
            key.interestOps(key.interestOps() & ~SelectionKey.OP_WRITE);
        }
    }

    void socketRead(SelectionKey key) throws IOException {
        // get the socket
        SocketChannel socket = (SocketChannel) key.channel();
        // read the data from the socket into the readbuf
        socket.read(readBuf);

        // set the read buf to reading mode
        readBuf.flip();
        parseMessage(readBuf);
        readBuf.compact();
    }

    public void parseMessage(ByteBuffer buf) {
        // check and see if we have a valid event
        // will return null if we dont have a full event yet
        RawEvent event = RawEvent.decode(readBuf);

        if (event != null) {
            // send the created event to the read queue
            boolean res = toReadQueue.offer(event);
            if (!res) {
                throw new RuntimeException("Unable to offer to client read queue");
            }
            // if we made an event and there is still data remaining that means that there
            // might be a second event in the buffer that we parse so we call parseMessage
            if (readBuf.hasRemaining()) {
                parseMessage(buf);
            }
        }

    }

    public void write(RawEvent event) {
        // if not initalized then we write to the startUpQueue
        if (!isConnected.get()) {
            // add the event to the queue to be handled later
            startUpQueue.add(event);
        } else if (ensureKeyValid(key)) {
            // create the buf
            ByteBuffer buf = RawEvent.encode(event);
            // add the buf to the queue
            writeQueue.add(buf);
            // set the key, telling the selector to write, this tells the selector that I
            // want to write on this channel that the key belongs to
            key.interestOps(key.interestOps() | SelectionKey.OP_WRITE);
            // this wakes up the selector and tells it to do something
            key.selector().wakeup();
        }
    }

    boolean ensureKeyValid(SelectionKey key) {
        if (key.isValid()) {
            return true;
        } else {
            invalidateKey();
            return false;
        }
    }

    void invalidateKey() {
        disconnect();
        onDisconnect.run();
    }

    public LinkedBlockingQueue<RawEvent> readQueue() {
        return toReadQueue;
    }

    // copied from server
    void resolveCancelledKey(SelectionKey key) throws IOException {
        if (key == null)
            throw new RuntimeException("Trying to close a null key");

        // cancel key
        key.cancel();

        // close socket
        SocketChannel socket = (SocketChannel) key.channel();

        if (socket != null) {
            socket.close();
        }
    }

    public void disconnect() {
        try {
            this.isConnected.setRelease(false);
            if (socket != null) socket.close();
            selector.close();
            this.thread.interrupt();
        } catch (IOException e) {
            System.out.println("resolving key failed");
            System.out.println(e);
        }
    }

    public AtomicBoolean getConnected() {
        return this.isConnected;
    }
}
