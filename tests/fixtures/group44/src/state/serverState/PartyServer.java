package state.serverState;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import javax.swing.Timer;

import eventSystem.engine.ServerEventEngine;
import eventSystem.events.Event;
import state.clientState.GameState;
import state.clientState.Player;
import state.clientState.events.JoinEvent;
import state.clientState.events.IntoTyperacerEvent;
import state.clientState.events.DisconnectEvent;
import state.clientState.events.IntoRankingEvent;
import state.clientState.lobbyState.LobbyJoinEvent;
import state.clientState.lobbyState.LobbyReadyEvent;
import state.clientState.lobbyState.PlayerLobbyState;
import state.clientState.rankState.PlayerRankingState;
import state.clientState.rankState.RankReadyEvent;
import state.clientState.rankState.Ranking;
import state.clientState.rankState.RankRestartToggleEvent;
import state.clientState.typeracerState.PlayerTyperacerState;
import state.clientState.typeracerState.TyperacerStatusEvent;
import state.clientState.typeracerState.TyperacerTypeEvent;
import state.clientState.typeracerState.PlayerTyperacerState.PlayerStatus;
import state.clientState.typeracerState.TyperacerState;
import state.serverState.paragraphGenerator.Paragraph;
import state.serverState.paragraphGenerator.ParagraphGenerator;
import utils.Tuple;

/**
 * PartyServer
 * 
 * @author Lucas Vanderwielen
 */
public class PartyServer extends ServerEventEngine {
    final static String[] COLORS = { "pink", "green", "yellow", "purple", "orange" };

    final static long startTimerLength = 1000;
    final static int MAX_GAMES_PLAYED = 3;

    HashMap<String, Player> players;
    GameState state;
    ParagraphGenerator gen;
    Paragraph par;

    HashMap<String, PlayerLobbyState> lobbyState;
    HashMap<String, Tuple<PlayerTyperacerState, ServerPlayerTypeState>> typeState;
    HashMap<String, PlayerRankingState> rankState;
    boolean isRestartMode;

    HashMap<String, Integer> clientIdMap;

    long typeracerStartTime;

    int gamesPlayed;
    int difficulty;

    Timer rankingReadyTimer;
    Timer lobbyReadyTimer;
    Timer typeracerOverTimer;

    Timer typeracerGameTimer;

    public PartyServer(Player me, int port, int difficulty) {
        super(port);

        this.players = new HashMap<>();
        this.clientIdMap = new HashMap<>();

        this.state = GameState.Lobby;

        this.gen = new ParagraphGenerator();
        this.difficulty = difficulty;
        this.gamesPlayed = 0;

        initLobbyState();
    }

    void initLobbyState() {
        this.lobbyState = new HashMap<>();

        for (Map.Entry<String, Player> p : players.entrySet()) {
            PlayerLobbyState s = new PlayerLobbyState(p.getKey());

            this.lobbyState.put(p.getKey(), s);
        }
        this.state = GameState.Lobby;
    }

    void initTyperacerState() {
        this.typeracerOverTimer = null;
        this.typeState = new HashMap<>();

        for (Map.Entry<String, Player> p : players.entrySet()) {

            PlayerTyperacerState s = new PlayerTyperacerState(p.getKey());
            ServerPlayerTypeState ss = new ServerPlayerTypeState(p.getKey());
            Tuple<PlayerTyperacerState, ServerPlayerTypeState> t = new Tuple<PlayerTyperacerState, ServerPlayerTypeState>(
                    s,
                    ss);

            this.typeState.put(p.getKey(), t);
        }
        this.state = GameState.Typeracer;
    }

    void initRankingState() {
        this.rankState = new HashMap<>();
        for (Map.Entry<String, Player> p : players.entrySet()) {

            PlayerRankingState s = new PlayerRankingState(p.getKey());
            this.rankState.put(p.getKey(), s);
        }

        this.isRestartMode = false;
        this.state = GameState.Ranking;
    }

    @Override
    public Event handleEvent(int clientId, Event event) {
        switch (this.state) {
            case Lobby:
                return handleLobbyEvent(clientId, event);
            case Typeracer:
                return handleTyperacerEvent(clientId, event);
            case Ranking:
                return handleRankingEvent(clientId, event);
            default:
        }

        return null;
    }

    @Override
    public void handleClientDisconnect(int clientId) {
        String stringId = null;
        for (Map.Entry<String, Integer> v : clientIdMap.entrySet()) {
            if (v.getValue() == clientId) {
                stringId = v.getKey();
            }
        }

        if (stringId == null) {
            throw new RuntimeException("Unable to remove player becuase player does not exist, ClientId " + clientId);
        }

        Player p = players.get(stringId);

        p.disconnect();
        System.out.println("Client Disconnected...");

        broadcastNot(clientId, new DisconnectEvent(stringId));

        checkStatus();
    }

    void checkStatus() {
        if (state == GameState.Lobby) {
            checkAllReadyLobby();
        } else if (state == GameState.Typeracer) {
            checkTyperacerOver();
        } else if (state == GameState.Ranking) {
            checkAllReadyRanking();
        }

    }

    void checkAllReadyLobby() {
        cancelLobbyReadyTimer();
        boolean allReady = true;
        for (Map.Entry<String, PlayerLobbyState> s : lobbyState.entrySet()) {
            Player p = players.get(s.getKey());
            if (!s.getValue().getReady() && p.isConnected()) {
                allReady = false;
            }
        }
        if (allReady) {
            lobbyReadyTimer = new Timer(1000, (e) -> {
                startTyperacer();
            });
            lobbyReadyTimer.setRepeats(false);
            lobbyReadyTimer.start();

        }
    }

    void checkAllReadyRanking() {
        if (isMaxGamesPlayed()) {
            return;
        }
        cancelRankingReadyTimer();

        boolean allReady = true;
        for (Map.Entry<String, PlayerRankingState> s : rankState.entrySet()) {
            Player p = players.get(s.getKey());

            if (!s.getValue().getReady() && p.isConnected()) {
                allReady = false;
            }
        }
        if (allReady) {
            rankingReadyTimer = new Timer(5000, (e) -> {
                endRanking();
            });
            rankingReadyTimer.setRepeats(false);
            rankingReadyTimer.start();
        }
    }

    void checkTyperacerOver() {
        boolean finished = true;
        for (Map.Entry<String, Tuple<PlayerTyperacerState, ServerPlayerTypeState>> s : typeState.entrySet()) {
            Tuple<PlayerTyperacerState, ServerPlayerTypeState> state = s.getValue();
            Player p = players.get(s.getKey());
            if (state.first.getStatus() == PlayerStatus.Playing && p.isConnected()) {
                finished = false;
            }
        }

        if (finished && typeracerOverTimer == null) {
            typeracerOverTimer = new Timer(5000, (e) -> {
                endTyperacer();
            });
            typeracerOverTimer.setRepeats(false);
            typeracerOverTimer.start();
        }
    }

    void endRanking() {

        startTyperacer();
    }

    void endTyperacer() {
        initRankingState();
        this.typeracerGameTimer.stop();
        this.typeracerGameTimer = null;
        this.isRestartMode = false;

        List<Ranking> rankingList = new ArrayList<>();
        for (Map.Entry<String, Player> p : players.entrySet()) {

            Tuple<PlayerTyperacerState, ServerPlayerTypeState> tuple = typeState.get(p.getKey());
            PlayerTyperacerState state = tuple.first;
            ServerPlayerTypeState serverState = tuple.second;
            int playerPoints = serverState.points;

            double progress = (double) state.getPosition() / (double) par.getParagraph().length();

            long completedTime = serverState.completed ? serverState.completedTime - typeracerStartTime
                    : Long.MAX_VALUE;

            Ranking r = new Ranking(p.getKey(), progress, completedTime, serverState.completed,
                    serverState.peakWPM,
                    playerPoints);
            rankingList.add(r);
        }

        rankingList = rankingList.stream().sorted((a, b) -> {
            return a.points() <= b.points() ? 1 : -1;
        }).toList();

        Ranking[] arr = rankingList.toArray(new Ranking[0]);
        PlayerRankingState[] rankState = this.rankState.values().toArray(new PlayerRankingState[0]);

        broadcast(new IntoRankingEvent(arr, rankState, difficulty + 1, gamesPlayed >= MAX_GAMES_PLAYED));
    }

    boolean isMaxGamesPlayed() {
        return gamesPlayed >= MAX_GAMES_PLAYED;
    }

    void startTyperacer() {
        initTyperacerState();

        PlayerTyperacerState[] arr = this.typeState.values().stream().map(t -> t.first).toList()
                .toArray(new PlayerTyperacerState[0]);

        typeracerStartTime = System.currentTimeMillis() + startTimerLength;

        if (!this.isRestartMode) {
            this.gamesPlayed += 1;
            this.par = this.gen.getParagraph(difficulty);
            difficulty += 1;
        }

        broadcast(new IntoTyperacerEvent(arr, typeracerStartTime, par.getParagraph(), difficulty));

        int dur = (int) (startTimerLength + TyperacerState.DURATION);
        typeracerGameTimer = new Timer(dur, (e) -> {
            endTyperacer();
        });
        typeracerGameTimer.start();
    }

    Event handleTyperacerEvent(int clientId, Event event) {
        if (event instanceof TyperacerTypeEvent) {
            // casting the event
            TyperacerTypeEvent e = (TyperacerTypeEvent) event;
            // getting the player start
            Tuple<PlayerTyperacerState, ServerPlayerTypeState> t = typeState.get(e.getPlayerId());
            // get the client player state
            PlayerTyperacerState s = t.first;
            // get the server client
            ServerPlayerTypeState ss = t.second;

            // set the new position in the client state
            s.setPosition(e.getPosition());

            // setting the points
            ss.points = e.getPoints();

            // set the server peak WPM
            ss.peakWPM = e.getPeakWPM();

            if (s.getPosition() == par.getParagraph().length()) {
                ss.complete(event.createTime());
            }

            broadcastNot(clientId, e);

            checkTyperacerOver();
        } else if (event instanceof TyperacerStatusEvent) {
            TyperacerStatusEvent e = (TyperacerStatusEvent) event;

            Tuple<PlayerTyperacerState, ServerPlayerTypeState> s = typeState.get(e.getId());
            s.first.setStatus(e.getStatus());

            broadcastNot(clientId, e);

            checkTyperacerOver();
        }

        return null;
    }

    Event handleRankingEvent(int clientId, Event event) {

        if (event instanceof RankReadyEvent) {
            RankReadyEvent e = (RankReadyEvent) event;

            PlayerRankingState s = rankState.get(e.getPlayerId());

            s.setReady(e.getReady());

            broadcastNot(clientId, e);

            checkAllReadyRanking();
        } else if (event instanceof RankRestartToggleEvent) {
            RankRestartToggleEvent e = (RankRestartToggleEvent) event;
            this.isRestartMode = !this.isRestartMode;
            broadcastNot(clientId, e);
        }
        return null;
    }

    Event handleLobbyEvent(int clientId, Event event) {
        if (event instanceof LobbyReadyEvent) {
            LobbyReadyEvent e = (LobbyReadyEvent) event;
            PlayerLobbyState p = lobbyState.get(e.getPlayerId());

            p.setReady(e.getReadyState());

            broadcastNot(clientId, e);

            checkAllReadyLobby();

        } else if (event instanceof JoinEvent) {
            JoinEvent e = (JoinEvent) event;
            Player p = e.getPlayer();
            String id = p.id;
            p.setColor(COLORS[players.size() % 5]);

            lobbyState.put(id, new PlayerLobbyState(id));
            players.put(p.id, p);

            clientIdMap.put(p.id, clientId);

            Player[] pList = players.values().toArray(new Player[0]);

            PlayerLobbyState[] lList = lobbyState.values().toArray(new PlayerLobbyState[0]);

            broadcastNot(clientId, new LobbyJoinEvent(id, p));
            cancelLobbyReadyTimer();

            return new JoinEvent.JoinEventRes(pList, lList);
        }

        return null;

    }

    void cancelLobbyReadyTimer() {
        if (lobbyReadyTimer != null) {
            lobbyReadyTimer.stop();
            lobbyReadyTimer = null;
        }
    }

    void cancelRankingReadyTimer() {
        if (rankingReadyTimer != null) {
            rankingReadyTimer.stop();
            rankingReadyTimer = null;
        }
    }
}
