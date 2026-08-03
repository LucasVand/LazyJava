package gameplayScreens;

import javax.swing.JPanel;

import broadcasting.RoomServer;
import controller.game.GameController;
import state.clientState.ClientState;

/**
 * The MainGame class acts as the core container and entry point for an active
 * game session.
 * It extends JPanel (serving as a potential graphical container, though
 * primarily used here
 * to link UI with logic) and holds the foundational references to the client's
 * network state
 * and the main game loop controller.
 * 
 * @author Sam Deitz
 * @see controller.game.GameController
 * @see state.clientState.ClientState
 */
public class MainGame extends JPanel {

    /** The central controller responsible for managing game logic and flow. */
    private GameController controller;
    /**
     * The state object containing all networked data, player info, and game data
     * for this client.
     */
    private ClientState cState;

    /**
     * Constructs a new MainGame instance, establishing the client's network state
     * and initializing the primary game controller.
     *
     * @param ip         The IP address of the server to connect to.
     * @param host       True if this local client is also the host of the game,
     *                   false otherwise.
     * @param difficulty The selected difficulty level for the typing race.
     * @param server     The RoomServer instance (only relevant/non-null if this
     *                   client is the host).
     */
    public MainGame(String ip, boolean host, int difficulty, RoomServer server) {
        // CREATE CLIENT STATE AND PASS ITSELF TO GAME CONTROLLER

        // Initialize the client state, attempting to connect to the provided IP on port
        // 5001
        cState = new ClientState(ip, 5001, host, difficulty - 1);

        // Instantiate the GameController, passing it the newly created state,
        // a reference to this MainGame panel, and the server instance (if hosting)
        controller = new GameController(cState, this, server);
    }

    /**
     * Retrieves the game controller managing the active session's logic.
     *
     * @return The current GameController instance.
     */
    public GameController getController() {
        return controller;
    }

    /**
     * Retrieves the network and game state for the current client.
     *
     * @return The current ClientState instance.
     */
    public ClientState getState() {
        return cState;
    }
}
