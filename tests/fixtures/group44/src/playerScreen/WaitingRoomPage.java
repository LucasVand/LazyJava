package playerScreen;

import java.awt.AlphaComposite;
import java.awt.BorderLayout;
import java.awt.Component;
import java.awt.Dimension;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.image.BufferedImage;
import java.util.ArrayList;

import javax.imageio.ImageIO;
import javax.swing.Box;
import javax.swing.JPanel;
import javax.swing.SwingUtilities;

import UIComponents.Flexbox;
import UIComponents.PageLayout;
import UIComponents.StyledButton;
import UIComponents.Text;
import broadcasting.RoomServer;
import controller.screens.screenEvent.ScreenEventBus;
import state.clientState.ClientState;
import state.clientState.GameState;
import state.clientState.Player;
import state.clientState.lobbyState.PlayerLobbyState;
import utils.ColorManager;
import utils.Tuple;

public class WaitingRoomPage extends JPanel {

    /**
     * Amount of players ready to play
     */
    private int playersReady = 0;

    /**
     * Broadcasting server for the room (IF THIS IS THE HOST)
     */
    private RoomServer server;

    /**
     * State of this player's client
     */
    private ClientState cState;

    /**
     * States of all players in the lobby
     */
    private Tuple<Player, PlayerLobbyState>[] states;

    /**
     * State of the host
     */
    private Tuple<Player, PlayerLobbyState> hostState;

    /**
     * List of states of all players that are not the host
     */
    private ArrayList<Tuple<Player, PlayerLobbyState>> joinedPlayers = new ArrayList<>();

    /**
     * Flexbox containing all player icons
     */
    private Flexbox people = new Flexbox(true);

    /**
     * Text indicating how many players are ready to play
     */
    private Text ready_text;

    /**
     * All colors for player boats
     */
    private String[] colors = { "pink", "green", "yellow", "purple", "orange", "brown" };

    /**
     * All sprites for boats
     */
    private BufferedImage[] boatSprites = new BufferedImage[colors.length];

    /**
     * Background image
     */
    private BufferedImage waveBg;

    /**
     * Ready button
     */
    private StyledButton ready;

    /**
     * Initialize a new waiting room page
     * 
     * Set up initial UI and save the server (IF HOST). Load images for UI.
     * Set callbacks for state changes
     * 
     * @param cState State of this user's game server
     * @param server broadcasting server or null
     */
    public WaitingRoomPage(ClientState cState, RoomServer server) {

        // Set up page
        setPreferredSize(new Dimension(1200, 700));
        setFocusable(true);
        setLayout(new BorderLayout());
        setOpaque(false);
        people.setOpaque(false);
        this.server = server;

        try {
            waveBg = ImageIO.read(getClass().getResourceAsStream("/resources/images/waveBg.png"));
        } catch (Exception e) {
            System.err.println("Could not load background image!");
            e.printStackTrace();
        }

        // load boat images
        loadImages();

        // load state
        this.cState = cState;
        states = cState.getLobbyPlayerState();

        // create initial UI
        createButtonMenu();
        createPlayerIcons();

        // Ensure the first render happens AFTER the component is added to the parent
        SwingUtilities.invokeLater(() -> {
            updatePlayers();
            renderPlayers();
            updatePlayerCount();
            revalidate();
            repaint();
        });

        // callback for updating players based on ready status or join event
        cState.getLobbyState().setOnStateChange(() -> {
            this.updatePlayers();
            this.renderPlayers();
            updatePlayerCount();
        });
        setVisible(true);
    }

    /**
     * Update the amount of players in the room object
     */
    private void updatePlayerCount() {
        if (server != null) {
            server.getRoom().setPlayerCount(cState.getTotalPlayers());
        }
    }

    /**
     * Create the player icon UI menu
     */
    private void createPlayerIcons() {

        // Render the palayers to the people panel
        renderPlayers();

        // add it to the main panel
        add(people, BorderLayout.CENTER);
    }

    /**
     * Update the players in the game
     */
    public void updatePlayers() {

        // check if we are in the lobby state
        if (cState.getState() != GameState.Lobby) {
            return;
        }

        // get client state updates
        states = cState.getLobbyPlayerState();

        // Sort players
        hostState = null;
        joinedPlayers.clear();

        // sort states into host and other players
        for (Tuple<Player, PlayerLobbyState> state : states) {
            if (state != null && state.first != null) { // null check
                if (state.first.isHost) {
                    hostState = state;
                } else {
                    joinedPlayers.add(state);
                }
            }
        }

    }

    /**
     * Render players to icon menu
     */
    private void renderPlayers() {

        // clear menu and ready players
        people.removeAll();
        playersReady = 0;

        // Set up rows
        Flexbox hostRow = new Flexbox();
        hostRow.setOpaque(false);

        Flexbox topRow = new Flexbox();
        topRow.setOpaque(false);
        topRow.add(Box.createHorizontalGlue());

        Flexbox bottomRow = new Flexbox();
        bottomRow.setOpaque(false);
        bottomRow.add(Box.createHorizontalGlue());

        // Render the Host Row
        if (hostState != null) {
            Player p = hostState.first;
            // check if host is ready
            if (hostState.second.getReady())
                playersReady++;

            // Add row with spacing
            hostRow.add(Box.createHorizontalGlue());
            hostRow.add(new PlayerIcon(p.name, true, getBoatImage(p.color)));
            hostRow.add(Box.createHorizontalGlue());
            hostRow.add(Box.createHorizontalGlue());
        }

        // Render the Regular Players
        for (int i = 0; i < 4; i++) {
            PlayerIcon pi;

            // If we have a player for this slot, use their data
            if (i < joinedPlayers.size()) {
                Tuple<Player, PlayerLobbyState> pState = joinedPlayers.get(i);
                Player p = pState.first;

                // Check if the player is ready
                if (pState.second.getReady())
                    playersReady++;

                // create new player icon with player color
                pi = new PlayerIcon(p.name, false, getBoatImage(p.color));
            }

            // no player for this slot
            else {
                // create new empty player icon with default color
                pi = new PlayerIcon("Waiting...", false, getBoatImage("brown"));
            }

            // Distribute them: first 2 go top, next 2 go bottom
            if (i < 2) {
                topRow.add(pi);
                topRow.add(Box.createHorizontalGlue()); // Spacing
                topRow.add(Box.createHorizontalGlue());

            } else {
                bottomRow.add(pi);
                bottomRow.add(Box.createHorizontalGlue()); // Spacing
                bottomRow.add(Box.createHorizontalGlue());

            }
        }

        // add spacing
        topRow.add(Box.createHorizontalGlue());
        bottomRow.add(Box.createHorizontalGlue());

        // Add rows to panel
        people.add(Box.createVerticalGlue());
        people.add(topRow);
        people.add(Box.createVerticalGlue());
        people.add(hostRow);
        people.add(Box.createVerticalGlue());
        people.add(bottomRow);
        people.add(Box.createVerticalGlue());

        // update ready text
        ready_text.setText(playersReady + "/" + cState.getTotalPlayers() + " Players Ready");

        // repaint screen
        people.revalidate();
        people.repaint();
    }

    // Create button menu for right side of UI
    private void createButtonMenu() {
        // --- BACK BUTTON ---
        PageLayout.createBackButton(this, e -> {
            ScreenEventBus.publish("PLAYER_SCREEN", new PlayerScreenPage());
        });

        // --- RIGHT SIDE BUTTON PANEL ---
        Flexbox readyMenu = new Flexbox(true);
        readyMenu.setOpaque(false);
        readyMenu.addPadding(20);

        // Ready button
        ready = new StyledButton("Ready", StyledButton.ButtonStyle.PILL, ColorManager.primaryBlue,
                ColorManager.primarySand);
        ready.addActionListener(e -> handleReady());
        ready.setAlignmentX(Component.CENTER_ALIGNMENT);

        // Text for how many have said ready
        ready_text = new Text(playersReady + "/" + cState.getTotalPlayers() + " Players Ready", 15);
        ready_text.setAlignmentX(Component.CENTER_ALIGNMENT);
        ready_text.setForeground(ColorManager.primaryBrown);

        // Add to frame
        readyMenu.add(Box.createVerticalGlue());
        readyMenu.add(ready);
        readyMenu.add(Box.createVerticalStrut(5));
        readyMenu.add(ready_text);
        readyMenu.add(Box.createVerticalGlue());
        add(readyMenu, BorderLayout.EAST);
    }

    @Override
    public void paintComponent(Graphics g) {
        super.paintComponent(g);

        if (waveBg != null) {
            Graphics2D g2d = (Graphics2D) g.create();
            float opacity = 0.3f;
            g2d.setComposite(AlphaComposite.getInstance(AlphaComposite.SRC_OVER, opacity));
            g2d.drawImage(waveBg, 0, 0, this.getWidth(), this.getHeight(), null);
            g2d.dispose();
        }
    }

    /**
     * Handle when user clicks ready. Updates status in state and changes button UI
     */
    private void handleReady() {
        // toggle ready in client state
        cState.getLobbyState().toggleReady();

        // get the ready status of the user
        boolean isReady = cState.getMyLobbyState().second.getReady();

        // change button UI based on status
        if (isReady)
            ready.setNewColors(ColorManager.primarySand, ColorManager.primaryBlue);
        else
            ready.setNewColors(ColorManager.primaryBlue, ColorManager.primarySand);

        // rerender players and
        renderPlayers();
    }

    /**
     * Get a loaded boat image basaed on color
     * 
     * @param color color of desired image
     * @return boat image
     */
    private BufferedImage getBoatImage(String color) {

        // loop through array of images
        for (int i = 0; i < colors.length; i++) {

            // return if colors match
            if (color.equals(colors[i]))
                return boatSprites[i];
        }
        return null; // no boat with that color exists
    }

    /**
     * Load the boat images for use
     */
    private void loadImages() {

        for (int i = 0; i < colors.length; i++) {
            try {
                String imagePath = "/resources/images/" + colors[i] + "Boat.png";
                boatSprites[i] = ImageIO.read(getClass().getResourceAsStream(imagePath));
            } catch (Exception e) {
                System.err.println("Could not load boat image: " + colors[i] + "Boat.png");
                e.printStackTrace();
            }
        }
    }

    /**
     * Get the broadcast server if this is the host
     * 
     * @return broadcast server
     */
    public RoomServer getServer() {
        return server;
    }
}
