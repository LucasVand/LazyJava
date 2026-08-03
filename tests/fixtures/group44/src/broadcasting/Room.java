package broadcasting;

import java.io.Serializable;
import java.net.InetAddress;
import java.util.UUID;

/**
 * Class representing an open game room. This class will be broadcasted through UDP 
 * to allow players to connect.
 * 
 * @author Sam Deitz
 * @see broadcasting.RoomFinder
 * @see broadcasting.RoomServer
 */
public class Room implements Serializable {
    
    /**
     * The port the room is broadcasting through
     */
    private final int port;

    /**
     * The IP address the host is running on
     */
    private final String address;

    /**
     * Name of the host of the room
     */
    private final String hostName;

    /**
     * Unique ID of the room
     */
    private String id;

    /**
     * Timestamp for when the room was last recieved as a packet
     */
    private long lastRead;

    /**
     * Amount of players currently in the room
     */
    private int players = 1;


    /**
     * Creates an object containing room information sent over UDP
     * @param hostName name of the room
     * @param address IP address of the room 
     * @param port port to connect through
     */
    public Room(String hostName, InetAddress address, int port){
        this.port = port;
        this.address = address.toString().split("/")[1];
        this.hostName = hostName;
        id = UUID.randomUUID().toString();
    }

    /**
     * Creates an object containing room information sent over UDP
     * @param hostName name of the room
     * @param address IP address of the room 
     * @param port port to connect through
     */
    public Room(String hostName, String address, int port){
        this.port = port;
        this.address = address;
        this.hostName = hostName;
    }

    /**
     * Getter for host name
     * @return name of host
     */
    public String hostName() {
        return hostName;
    }

    /**
     * Getter for broadcasting port
     * @return broadcasting port
     */
    public int port() {
        return port;
    }

    /**
     * Getter for IP of host
     * @return IP address
     */
    public String address() {
        return address;
    }

    /**
     * Getter for unique ID of the room
     * @return ID
     */
    public String id() {
        return id;
    }

    /**
     * Getter for time last read by a finder
     * @return last read system time
     */
    public long lastRead() {
        return lastRead;
    }

    /**
     * Get the amount of players in the room
     * @return number of players
     */
    public int getPlayers() {
        return players;
    }

    /**
     * Set the amount of players in the room
     * @param x new amount
     */
    public void setPlayerCount(int x) {
        players = x;
    }

    /**
     * Update the last read instance variable
     */
    public void read() {
        lastRead = System.currentTimeMillis();
    }

    @Override
    public String toString() {
        return String.format("%s: %s - %d", hostName, address, port);
    }

}
