package playerScreen;

import java.awt.Dimension;

import javax.swing.Box;

import UIComponents.Flexbox;
import UIComponents.StyledButton;
import UIComponents.Subheader;
import broadcasting.Room;
import gameplayScreens.MainGame;
import utils.ColorManager;

/**
 * This object represents an open room card to show a joinable game room.
 * It will display the room name, amount of players in the room, and 
 * give a button to join.
 */
public class OpenRoom extends Flexbox {

    /**
     * Width of the card
     */
    final int WIDTH = 600;

    /**
     * Height of the card
     */
    final int HEIGHT = 70;

    /**
     * Room the card represents
     */
    private Room room;
     
    /**
     * Initialize a new OpenRoom card.
     * 
     * Displays name, players, and a join option
     * @param room room
     * @param maxPlayers max players for the room
     */
    public OpenRoom(Room room, int maxPlayers) {
        super();
        this.room = room;

        // setup component
        addPadding(20);
        setPreferredSize(new Dimension(WIDTH, HEIGHT));
        setMinimumSize(new Dimension(WIDTH, HEIGHT));
        setMaximumSize(new Dimension(WIDTH, HEIGHT));
        setVisible(true);
        setBackground(ColorManager.primaryBlue);
        setForeground(ColorManager.primarySand);

        // subcomponents for the panel
        Subheader name = new Subheader(room.hostName()); // room name
        Subheader lobbyStatus = new Subheader(room.getPlayers() + "/" + maxPlayers); // x/x players in the room
        StyledButton joinBtn = new StyledButton("Join", StyledButton.ButtonStyle.RECT); // Button to join the room 
        joinBtn.addActionListener(e -> handleJoin()); // action listener for button

        name.setForeground(ColorManager.primarySand);
        lobbyStatus.setForeground(ColorManager.primarySand);
        
        // add components
        add(name);
        add(Box.createHorizontalGlue());
        add(lobbyStatus);
        add(Box.createHorizontalStrut(15));
        add(joinBtn);
    }

    /**
     * Handles a player trying to join a room
     */
    private void handleJoin() {

        // initialize a new game for the user with the room's address
        new MainGame(room.address(), false, 1, null);
    }

}
