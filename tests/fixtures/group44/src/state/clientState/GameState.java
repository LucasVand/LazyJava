
package state.clientState;

/**
 * GameState, this is the current state of the game. Each state represents a
 * different page of the gameplay part of the game. Connecting is an inital
 * state when the client is still waiting for confimation that it has been
 * allowed into the room. Lobby is where the players are waiting to start the
 * game, players can ready up and see all the players currently in the room.
 * Typeracer this is where the actual game is played, players type a paragraph
 * as fast as they can and race against the other players. Ranking is where
 * players will view the stats from the previous mini game along with how other
 * players did.
 */
public enum GameState {
    Connecting,
    Lobby,
    Typeracer,
    Ranking
}
