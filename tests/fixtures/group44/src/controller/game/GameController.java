package controller.game;

import java.awt.CardLayout;

import javax.swing.JPanel;

import broadcasting.RoomServer;
import controller.Controller;
import controller.screens.screenEvent.ScreenEventBus;
import gameplayScreens.GameOver;
import gameplayScreens.Gameplay;
import playerScreen.PlayerScreenPage;
import playerScreen.WaitingRoomPage;
import state.clientState.ClientState;

/**
 * The GameController object controls screen flow for all gameplay states. This
 * controller is NOT a listener
 * as all flow is controlled from this class. It initializes a subcontroller
 * under ScreenController,
 * and posts one ScreenEvent to show on the main screen.
 * <p>
 * Under this class the subcontroller transitions
 * between the 4 states of the game based on the ClientState. It
 * contains variables for ClientState
 * and RoomServer to control closing and opening the game.
 * 
 * @author Sam Deitz
 * @see controller.Controller
 */
public class GameController implements Controller {

    /**
     * State of this client
     */
    private ClientState cState;

    /**
     * Server the room is broadcasting over (IF THIS IS THE HOST)
     */
    private RoomServer broadcastingServer;

    /**
     * Panel housing pages
     */
    private final JPanel cardContainer;

    /**
     * Card layout to manage which page is shown
     */
    private final CardLayout cardLayout;

    /**
     * Initialize a new controller for the gameplay screens (WaitingRoomPage,
     * Gameplay, GameOver)
     * 
     * This object will handle switching which screen is visible to the user
     * 
     * @param cState player state
     * @param game   game panel
     * @param server server the room is broadcasting over (IF HOST)
     */
    public GameController(ClientState cState, JPanel game, RoomServer server) {
        this.cState = cState;
        this.broadcastingServer = server;

        // initialize cardlayout for screen management
        cardContainer = game;
        cardLayout = new CardLayout();
        cardContainer.setLayout(cardLayout);

        // set callback to close the server if the host disconnects
        this.cState.setOnHostDisconnect(() -> {
            System.out.println("Host Disconnected");
            ScreenEventBus.publish("Player Screen", new PlayerScreenPage());
            cState.close();
        });

        // set callback for when the state changes between screens
        cState.setPageChangeCallback((state) -> {
            switch (state) {

                // connecting state
                case Connecting:
                    break;

                // user is in the lobby
                case Lobby:
                    // initialize waiting room and show it with the cardlayout
                    registerScreen("LOBBY", new WaitingRoomPage(cState, server));
                    showScreen("LOBBY");
                    break;
                case Ranking:
                    // initialize game over screen for ranking
                    registerScreen("GAMEOVER", new GameOver(cState));
                    showScreen("GAMEOVER");
                    break;
                case Typeracer:

                    // initialize racing screen and show it
                    registerScreen("TYPERACER", new Gameplay(cState));
                    showScreen("TYPERACER");

                    // close broadcast so no one can join mid game
                    if (server != null && server.isBroadcasting()) {
                        server.stopBroadcasting();
                    }
                    break;
                default:
                    break;

            }
        });

        // send the event to show the game screens to the main controller
        ScreenEventBus.publish("GAMING", game);
    }

    /**
     * Add the screen the CardLayout for GameController
     */
    @Override
    public void registerScreen(String screenName, JPanel screenPanel) {
        cardContainer.add(screenPanel, screenName);
    }

    /**
     * Show the screen in the cardlayout
     */
    @Override
    public void showScreen(String screenName) {
        // change screen in frame
        cardLayout.show(cardContainer, screenName);
    }

    /**
     * Getter for the broadcast server if this is the host
     * 
     * @return the server broadcasting room
     */
    public RoomServer getRoomServer() {
        return broadcastingServer;
    }

}
