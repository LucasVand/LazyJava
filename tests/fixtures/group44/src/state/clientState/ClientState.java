package state.clientState;

import java.util.Arrays;
import java.util.HashMap;
import java.util.UUID;
import java.util.function.Consumer;

import eventSystem.engine.ClientEventEngine;
import saveData.Account;
import saveData.SaveData;
import saveData.Stats;
import state.clientState.GameState;
import state.clientState.Player;
import state.clientState.events.JoinEvent;
import state.clientState.events.DisconnectEvent;
import state.clientState.events.IntoRankingEvent;
import state.clientState.events.IntoTyperacerEvent;
import state.clientState.events.JoinEvent.JoinEventRes;
import state.clientState.lobbyState.LobbyJoinEvent;
import state.clientState.lobbyState.LobbyState;
import state.clientState.lobbyState.PlayerLobbyState;
import state.clientState.rankState.PlayerRankingState;
import state.clientState.rankState.RankState;
import state.clientState.rankState.Ranking;
import state.clientState.typeracerState.*;
import state.serverState.PartyServer;
import utils.Tuple;

/**
 * This is the main interface between the server and client. Contains all logic
 * for interfacing with {@code PartyServer}. Handles all state changes and state
 * updates. The main mechanism is the {@code data}, this dynamically changes
 * what current data the Class is concerned with. When in the Lobby,
 * {@code LobbyState} is valid. When in Typeracer {@code TyperacerState} is
 * valid. When in Ranking then {@code RankState} is valid.
 *
 * *Usage*
 *
 * *Joining*
 * Creates a new client state which is not hosting the server. This will attempt
 * to connect to a server with the given ip and port.
 * {@snippet
 * ClientState state = new ClientState("10.0.0.138", 5001, false);
 * }
 *
 * *Hosting*
 * Creates a new client state which is hosting the server. This will attempt
 * to create a server with the given port and connect to it. Recommended you
 * leave the ip as "localhost" but a correct outward facing ip works as well
 * {@snippet
 * ClientState state = new ClientState("localhost", 5001, true);
 * }
 *
 * @author Lucas
 *
 */

public class ClientState {
    GameState state;
    GameData data;
    ClientEventEngine engine;
    Exception err;

    String me;

    HashMap<String, Player> players;
    boolean hosted;
    PartyServer server;

    Consumer<GameState> pageChangeCallback;

    Powerups powerups;
    int difficulty;

    /**
     * Main constructor for the {@code ClientState}, this can be used to create a
     * connection to the server
     *
     * @param ip     the IPv4 address of the server to connect or if hosting can be
     *               localhost
     * @param port   the port at which the server it bound to, this doubles as the
     *               port to open the server on if hosting
     * @param hosted whether the server should be hosted
     * @param difficulty difficluty of the game
     */
    public ClientState(String ip, int port, boolean hosted, int difficulty) {
        this.hosted = hosted;
        this.players = new HashMap<>();
        this.state = GameState.Connecting;

        powerups = new Powerups(2, 3, 2);
        Account account = SaveData.getData().getLoggedInAccount();

        Player me = new Player(account.getIdentity().getUsername(), account.getId(), hosted);
        this.me = me.id;
        this.players.put(me.id, me);
        this.difficulty = difficulty;

        // starts the server if we want to host
        if (hosted) {
            setServer(me, port);
        }

        // start the client event engine
        this.engine = new ClientEventEngine(ip, port, (_engine, e) -> {
            if (e == null) {
                System.out.println("Connected");
                return;
            }
            this.err = e;
            System.out.println("Failed to connect ClientState");
            System.out.println(e);
        });

        // // blocks while the client is still initalizing
        while (!this.engine.isConnected().get()) {
        }

        this.engine.sendEvent(new JoinEvent(me), (e) -> {
            this.handleJoin(e);
        });
        this.engine.addListener(LobbyJoinEvent.class, (e) -> {
            this.handleJoinEvent(e);
        });
        this.engine.addListener(DisconnectEvent.class, (e) -> {
            this.handleDisconnectEvent(e);
        });
        // set the listeners for the global transitions
        this.setGlobalEventListeners();

        // default page callback to prevent crashes
        this.pageChangeCallback = (e) -> {
        };
    }

    /**
     * sets the page callback. The page callback is called when the @{code
     * GameState} changes, when this changes it is expected that the UI page that is
     * shown should also change
     *
     * @param callback the callback which is called when the page changes
     */
    public void setPageChangeCallback(Consumer<GameState> callback) {
        this.pageChangeCallback = callback;
    }

    /**
     * sets the on host disconnect callback. This is called if the host ever
     * abruptly disconnects.
     *
     * @param callback the callback which will be called when the host leaves
     */
    public void setOnHostDisconnect(Runnable callback) {
        this.engine.setOnDisconnect(callback);
    }

    // starts the server
    void setServer(Player me, int port) {
        // create the new server
        this.server = new PartyServer(me, port, difficulty);
        this.server.start();

        // blocks while the server is still initalizing
        this.server.waitUntilStarted();

    }

    // gets global event listeners
    void setGlobalEventListeners() {
        this.engine.addListener(IntoTyperacerEvent.class, (e) -> {
            this.moveToTyperacer(e);
        });
        this.engine.addListener(IntoRankingEvent.class, (e) -> {
            this.moveFromTyperacerToRanking(e);
        });
    }

    // called when moving from typeracer to ranking
    void moveFromTyperacerToRanking(IntoRankingEvent e) {
        Player p = this.getPlayer();

        TyperacerState typeState = this.getTyperacerState();
        Stats s = typeState.getStats();
        long startTime = typeState.getStartTime();
        s.addPlayTime(System.currentTimeMillis() - startTime);
        s.setPoints(typeState.getPoints());
        s.completeMatch();

        p.getStats().combine(s);

        // if its the last then combine it with the player stats
        SaveData.getData().getLoggedInAccount().getStats().combine(p.getStats());

        this.data.drop();
        this.data = new RankState(e.getRankings(), e.getState(), e.getNextDifficulty(), e.isEnd(), me, this.engine);

        SaveData.getData().getHighscoreTable().updateScores(this.getPlayerRankings());
        SaveData.save();

        this.state = GameState.Ranking;
        this.pageChangeCallback.accept(this.state);

    }

    // move when moving into a new typeracer
    void moveToTyperacer(IntoTyperacerEvent e) {
        this.data.drop();
        this.data = new TyperacerState(e.getStates(), e.getParagraph(), e.getStartTime(), e.getDifficulty(),
                me,
                engine, powerups);

        this.state = GameState.Typeracer;
        this.pageChangeCallback.accept(this.state);
    }

    // handles the join event, this is when another player is joining the room
    void handleJoinEvent(LobbyJoinEvent e) {
        if (this.state != GameState.Lobby) {
            System.out.println("Cannot add player because not in lobby");
            return;
        }

        this.players.put(e.getPlayerId(), e.getPlayer());
        ((LobbyState) this.data).handleJoinEvent(e);
    }

    void handleDisconnectEvent(DisconnectEvent e) {
        Player p = this.players.get(e.getId());
        p.disconnect();

        if (data != null) {
            data.updateState();
        }
    }

    // this is called when we get the response from the server after sending a join
    // request event and we are reciving info about the status of the room along
    // with all the players in that room, this function then takes us into the Lobby
    // state
    void handleJoin(JoinEventRes e) {

        for (Player p : e.getPlayerList()) {
            this.players.put(p.id, p);
        }

        this.data = new LobbyState(e.getLobbyList(), me, this.engine);

        this.state = GameState.Lobby;
        this.pageChangeCallback.accept(this.state);

    }

    /**
     * whether the client is connected. When the client has been fully initialized
     * and listening this will be true.
     *
     * @return whether the client is initalized
     */
    public boolean connected() {
        return this.engine.isConnected().get();
    }

    /**
     * Get the current {@code GameState}. This is the current state of the game can
     * be Lobby, Ranking, Typeracer.
     *
     * @return the state of the game
     */
    public GameState getState() {
        return this.state;
    }

    /**
     * Get the error. This will be null if everything is normal. The main source of
     * this error will be the {@code ClientEventEngine}. If connection fails then
     * this will be the error that was thrown from the failed connection
     *
     * @return the execption thrown
     */
    public Exception getError() {
        return this.err;
    }

    /**
     * Get the current lobby state. This contains data related to the lobby. This
     * has all the players state in the lobby and allows the update of this players
     * state. This is guaranteed to be valid if the {@code GameState} is Lobby. If
     * the current game state is not Lobby calling this function will throw a
     * {@code ClassCastException} because {@code LobbyState} does not exist
     * 
     * @return the lobby state
     */
    public LobbyState getLobbyState() {
        return (LobbyState) this.data;
    }

    /**
     * Get the current rank state. This contains data related to the ranking page.
     * This has all the players state in the lobby and allows the update of this
     * players state. This is guaranteed to be valid if the {@code GameState} is
     * Ranking. If the current game state is not Ranking calling this function will
     * throw a {@code ClassCastException} because {@code RankState} does not exist
     * 
     * @return the rank state
     */
    public RankState getRankState() {
        return (RankState) this.data;
    }

    /**
     * Get the current typeracer state state. This contains data related to the
     * typeracer mini game. This has all the players state in the typeracer and
     * allows the update of this players state. This is guaranteed to be valid if
     * the {@code GameState} is Typeracer. If the current game state is not
     * Typeracer calling this function will throw a {@code ClassCastException}
     * because {@code TyperacerState} does not exist
     * 
     * @return the typeracer state
     */
    public TyperacerState getTyperacerState() {
        return (TyperacerState) this.data;
    }

    /**
     * Gets the player state in the typeracer setting. This returns a list of tuples
     * where the first value is {@code Player} which contains general information
     * about the player and second value which is {@code PlayerTyperacerState} which
     * contains typeracer specific information. This function will throw
     * {@code ClassCastException} if called when the {@code GameState} is not
     * Typeracer
     *
     * @return the player and typeracer states
     */
    @SuppressWarnings("unchecked")
    public Tuple<Player, PlayerTyperacerState>[] getTyperacerPlayerState() {
        TyperacerState r = getTyperacerState();
        return this.players.entrySet().stream().map(p -> {
            return new Tuple<Player, PlayerTyperacerState>(p.getValue(), r.getPlayerState(p.getKey()));
        }).toList().toArray(new Tuple[0]);
    }

    /**
     * Gets the players state in the ranking setting. This returns a list of tuples
     * where the first value is {@code Player} which contains general information
     * about the player and second value which is {@code PlayerRankingState} which
     * contains ranking specific information about the players. This function will
     * throw {@code ClassCastException} if called when the {@code GameState} is not
     * Ranking
     *
     * @return the player and ranking states
     */
    @SuppressWarnings("unchecked")
    public Tuple<Player, PlayerRankingState>[] getRankingPlayerState() {
        RankState r = getRankState();
        return this.players.entrySet().stream().map(p -> {
            return new Tuple<Player, PlayerRankingState>(p.getValue(), r.getPlayerState(p.getKey()));
        }).toList().toArray(new Tuple[0]);
    }

    /**
     * Gets the player state in the lobby setting. This returns a list of tuples
     * where the first value is {@code Player} which contains general information
     * about the player and second value which is {@code PlayerLobbyState} which
     * contains lobby specific information about the players. This function will
     * throw {@code ClassCastException} if called when the {@code GameState} is not
     * Lobby
     *
     * @return the player and lobby states
     */
    @SuppressWarnings("unchecked")
    public Tuple<Player, PlayerLobbyState>[] getLobbyPlayerState() {
        LobbyState r = getLobbyState();
        return this.players.entrySet().stream().map(p -> {
            return new Tuple<Player, PlayerLobbyState>(p.getValue(), r.getPlayerState(p.getKey()));
        }).toList().toArray(new Tuple[0]);
    }

    /**
     * Gets the current players ranking in the most recent mini game when the
     * {@code GameState} is Ranking. Gets a tuple with the first value
     * {@code Player} which is general player info and the second value which is
     * {@code Ranking} which contains information about the rankings from the last
     * mini game. This throws a {@code ClassCastException} if {@Code GameState} is
     * not Ranking
     *
     */
    @SuppressWarnings("unchecked")
    public Tuple<Player, Ranking>[] getPlayerRankings() {
        RankState state = getRankState();
        return Arrays.stream(state.getRankings()).map(r -> {
            return new Tuple<Player, Ranking>(this.players.get(r.playerId()), r);
        }).toList().toArray(new Tuple[0]);
    }

    /**
     * Gets a single players ranking in the most recent mini game when the
     * {@code GameState} is Ranking. Gets a tuple with the first value
     * {@code Player} which is general player info and the second value which is
     * {@code Ranking} which contains information about the rankings from the last
     * mini game. This throws a {@code ClassCastException} if {@Code GameState} is
     * not Ranking
     * 
     * @param id id
     */
    public Tuple<Player, Ranking> getPlayerRankings(String id) {
        RankState state = getRankState();
        for (Ranking r : state.getRankings()) {
            Player player = players.get(id);
            if (r.playerId().equals(id)) {
                return new Tuple<Player, Ranking>(player, r);
            }
        }
        return null;
    }

    /**
     * Gets the current players state in a lobby setting. Gets a tuple where the
     * first value is {@code Player} and the second value is
     * {@code PlayerLobbyState}. This throws {@code ClassCastException} if the
     * current {@code GameState} is not Lobby
     * 
     * @return lobby state
     */
    public Tuple<Player, PlayerLobbyState> getMyLobbyState() {
        LobbyState l = this.getLobbyState();
        return new Tuple<Player, PlayerLobbyState>(players.get(me), l.getPlayerState(me));
    }

    /**
     * Gets the current players state in a rank setting. Gets a tuple where the
     * first value is {@code Player} and the second value is
     * {@code PlayerRankingState}. This throws {@code ClassCastException} if the
     * current {@code GameState} is not Ranking
     * 
     * @return rank state
     */
    public Tuple<Player, PlayerRankingState> getMyRankState() {
        RankState l = this.getRankState();
        return new Tuple<Player, PlayerRankingState>(players.get(me), l.getPlayerState(me));
    }

    /**
     * Gets the current players state in a typeracer setting. Gets a tuple where the
     * first value is {@code Player} and the second value is
     * {@code PlayerTyperacerState}. This throws {@code ClassCastException} if the
     * current {@code GameState} is not Typeracer
     * 
     * @return typeracer state
     */
    public Tuple<Player, PlayerTyperacerState> getMyTyperacerState() {
        TyperacerState l = this.getTyperacerState();
        return new Tuple<Player, PlayerTyperacerState>(players.get(me), l.getPlayerState(me));
    }

    /**
     * Gets the current players general player object
     * 
     * @retrun this player
     */
    public Player getPlayer() {
        return this.players.get(me);
    }

    /**
     * This can be used to test whether a string id is the current players id. This
     * can be used as a helper function for testing whether state is the current
     * players
     *
     * @param id the id which to check
     *
     * @return whether the id is the current players id
     */
    public boolean isMe(String id) {
        return id.equals(me);
    }

    /**
     * Get the powerup object for the current player, this is global across all mini
     * games
     * 
     * @return the powerup object
     */
    public Powerups getPowerups() {
        return powerups;
    }

    /**
     * This should be called when the object is to be destroyed. This will close the
     * connection to the server and stop the server if it is being hosted
     */
    public void close() {
        this.engine.close();
        if (this.server != null) {
            this.server.close();
        }
    }

    /**
     * Gets the underlying engine that drives the event system, this should not be
     * tampered with use the exposed functions in this class to send and modify
     * state. Exposed for tests
     */
    public ClientEventEngine getEngine() {
        return this.engine;
    }

    public int getTotalPlayers() {
        return (int) players.entrySet().stream().filter(p -> {
            return p.getValue().isConnected();
        }).count();
    }
}
