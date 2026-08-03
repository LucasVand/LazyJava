package broadcasting;

import java.io.ByteArrayInputStream;
import java.io.ObjectInputStream;
import java.net.DatagramPacket;
import java.net.DatagramSocket;
import java.net.InetAddress;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.Iterator;

/**
 * Listens for room broadcasts over UDP. Uses separate threads to listen for
 * broadcasts, and cleanup old rooms that are no longer broadcasting. It
 * provides
 * methods to get the current list of open rooms, and close the thread.
 * 
 * @author Sam Deitz
 * @see broadcasting.Room
 */
public class RoomFinder {

    /**
     * Controls weather the thread is looking for rooms or not
     */
    private boolean findingRooms = false;

    /**
     * Hash map with id's and rooms found
     */
    private HashMap<String, Room> roomsFound = new HashMap<>();

    /**
     * Thread for reading packets
     */
    private final Thread reader;

    /**
     * Thread for cleaning up old rooms no longer being broadcasted
     */
    private final Thread cleanup;

    /**
     * Socket the thread is listening for rooms on
     */
    private DatagramSocket socket;

    /**
     * Constructor to create a room finder object. Starts two threads:
     * - The listener: Listens for room broadcasts
     * - Cleanup: Removes closed rooms
     */
    public RoomFinder() {
        findingRooms = true;

        // Listen for UDP broadcasts to find open rooms
        reader = new Thread(() -> {
            try {

                // initialize socket
                socket = new DatagramSocket(8888, InetAddress.getByName("0.0.0.0"));

                // set up buffer for recieving
                byte[] responseBuffer = new byte[2048];
                DatagramPacket responsePacket = new DatagramPacket(responseBuffer, responseBuffer.length);

                // continuously accept packets
                while (findingRooms) {

                    // recieve packet
                    socket.receive(responsePacket);

                    // read packet
                    try (ByteArrayInputStream byteInput = new ByteArrayInputStream(responsePacket.getData(), 0,
                            responsePacket.getLength());
                            ObjectInputStream objectInput = new ObjectInputStream(byteInput);) {

                        Room r = (Room) objectInput.readObject(); // convert packet to room object

                        roomsFound.put(r.id(), r); // add room
                        roomsFound.get(r.id()).read(); // reset the timestamp for the room broadcast
                    } catch (Exception e) {
                        System.out.println("Error receiving package.");
                    }
                }
            } catch (Exception e) {
                System.out.println(e);
            }
        }, "Room Finder");

        // Clean up rooms that are no longer broadcasting
        cleanup = new Thread(() -> {
            long timeout = 3000; // room must have been received within the last 3 seconds

            while (true) {
                try {
                    Iterator<String> it = roomsFound.keySet().iterator();

                    // loop through rooms
                    while (it.hasNext()) {
                        String id = it.next();

                        // room broadcast was not received in the last 3 seconds
                        if ((System.currentTimeMillis() - roomsFound.get(id).lastRead()) > timeout) {
                            it.remove(); // remove
                        }

                    }
                    Thread.sleep(1000);

                } catch (Exception e) {
                    System.out.println(e);
                }

            }
        }, "Remove closed rooms");

        // start threads
        reader.start();
        cleanup.start();
    }

    /**
     * Find all broadcasting rooms over the LAN
     * 
     * @return ArrayList of rooms
     */
    public ArrayList<Room> findRooms() {
        ArrayList<Room> rooms = new ArrayList<>();
        Iterator<Room> it = roomsFound.values().iterator();
        while (it.hasNext()) {
            Room r = it.next();
            rooms.add(r);
        }
        return rooms;
    }

    /**
     * Stops threads and closes socket
     */
    public void stopSearching() {
        findingRooms = false;
        cleanup.interrupt();
        if (socket != null && !socket.isClosed()) {
            socket.close(); // This breaks the blocking receive() call
        }
    }
}
