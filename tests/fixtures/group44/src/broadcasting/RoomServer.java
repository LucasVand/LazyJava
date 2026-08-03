package broadcasting;

import java.io.ByteArrayOutputStream;
import java.io.ObjectOutputStream;
import java.net.DatagramPacket;
import java.net.DatagramSocket;
import java.net.InetAddress;
import java.net.NetworkInterface;
import java.net.SocketException;
import java.util.Enumeration;

/**
 * This class is used to broadcast a room over UDP for other players to join.
 * It uses a separate thread to broadcast. It contains methods to stop the
 * server
 * and find the correct IP to broadcast.
 * 
 * @author Sam Deitz
 * @see broadcasting.Room
 */
public class RoomServer {

    /**
     * Controls if the server is broadcasting
     */
    private volatile boolean broadcasting = false;

    /**
     * Name of the room being broadcasted
     */
    private final String roomName;

    /**
     * Room being broadcasted
     */
    private Room r;

    /**
     * Socket the room is being broadcasted over
     */
    private DatagramSocket socket;

    /**
     * Thread broadcasting the room
     */
    private Thread broadcastThread;

    /**
     * Initialize a new room server
     * 
     * @param roomName name of the room
     */
    public RoomServer(String roomName) {
        this.roomName = roomName;
    }

    /**
     * Starts broadcasting room information for those on the LAN to join
     */
    public void startBroadcasting() {

        // thread for broadcasting
        broadcastThread = new Thread(() -> {
            try {
                // create socket
                socket = new DatagramSocket();
                socket.setBroadcast(true);

                InetAddress addr = getFirstNonLoopbackAddress(true, false);

                String ip = addr.getHostAddress();
                ip = ip.substring(0, ip.length() - 3);
                ip += "255";

                // create room to broadcast
                r = new Room(roomName, getFirstNonLoopbackAddress(true, false), 8888);
                broadcasting = true;

                // broadcasting loop
                while (broadcasting && !Thread.currentThread().isInterrupted()) {

                    // construct byte stream
                    ByteArrayOutputStream byteStream = new ByteArrayOutputStream();
                    ObjectOutputStream objectStream = new ObjectOutputStream(byteStream);

                    objectStream.writeObject(r);
                    objectStream.flush();

                    byte[] buffer = byteStream.toByteArray();

                    // construct packet
                    InetAddress broadcastAddress = InetAddress.getByName(ip);
                    DatagramPacket packet = new DatagramPacket(buffer, buffer.length, broadcastAddress, 8888);

                    // sent broadcast and sleep
                    socket.send(packet);
                    Thread.sleep(1000);
                }

            } catch (Exception e) {
                System.out.print(e);
            } finally {
                // This guarantees the socket closes even if the thread crashes
                if (socket != null && !socket.isClosed()) {
                    socket.close();
                    System.out.println("Broadcast socket forcefully closed in finally block.");
                }
            }
        });
        broadcastThread.start(); // start broadcast
    }

    /**
     * Closes the broadcast
     */
    public void stopBroadcasting() {
        broadcasting = false; // Break the while loop

        if (broadcastThread != null && broadcastThread.isAlive()) {
            broadcastThread.interrupt(); // Instantly wake it up from Thread.sleep(1000)
        }

        if (socket != null && !socket.isClosed()) {
            socket.close(); // Forcefully close the socket to release resources
        }
    }

    /**
     * Get the room being broadcasted
     * 
     * @return room being broadcasted
     */
    public Room getRoom() {
        return r;
    }

    /**
     * Determine if the server is broadcasting
     * 
     * @return true or false
     */
    public boolean isBroadcasting() {
        return broadcasting;
    }

    /**
     * Gives the address of the host.
     * 
     * This method ensures the host address is correct for windows and mac
     * 
     * @param preferIpv4 ipv4 address preferred over ipv6
     * @param preferIPv6 ipv6 address preferred over ipv4
     * @return address of the host
     * @throws SocketException there is a problem finding the address
     */
    public static InetAddress getFirstNonLoopbackAddress(boolean preferIpv4, boolean preferIPv6)
            throws SocketException {
        Enumeration<NetworkInterface> networkInterfaces = NetworkInterface.getNetworkInterfaces();
        while (networkInterfaces.hasMoreElements()) {
            NetworkInterface networkInterface = networkInterfaces.nextElement();

            // filters out 127.0.0.1 and inactive interfaces
            if (networkInterface.isLoopback() || !networkInterface.isUp())
                continue;

            Enumeration<InetAddress> addresses = networkInterface.getInetAddresses();
            while (addresses.hasMoreElements()) {
                InetAddress addr = addresses.nextElement();

                // Ensure it's not a loopback or link-local address
                if (!addr.isLoopbackAddress() && !addr.isLinkLocalAddress()) {
                    boolean isIPv4 = addr instanceof java.net.Inet4Address;
                    boolean isIPv6 = addr instanceof java.net.Inet6Address;

                    if (preferIpv4 && isIPv4) {
                        return addr;
                    } else if (preferIPv6 && isIPv6) {
                        return addr;
                    }
                }
            }
        }
        return null;
    }
}
