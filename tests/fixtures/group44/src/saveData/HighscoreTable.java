package saveData;

import java.io.Serializable;
import java.util.Comparator;
import java.util.HashMap;
import java.util.Map;
import java.util.stream.Collectors;

import state.clientState.Player;
import state.clientState.rankState.Ranking;
import utils.Tuple;

/**
 * HighscoreTable, stores all the highscore data for the overall highscores
 * page. Scores are unqiue to players so one player can only have one score in
 * the leaderboard. a max of 10 scores are held at once and sorted by score
 * highest being the best
 */
public class HighscoreTable implements Serializable {
    public final int MAX_LENGTH = 10;

    HashMap<String, Highscore> highscores;

    // Comparator function
    static final Comparator<Map.Entry<String, Highscore>> sorter = (h1, h2) -> {
        return h1.getValue().getScore() > h2.getValue().getScore() ? -1 : 1;
    };

    public HighscoreTable() {
        this.highscores = new HashMap<>();
    }

    /**
     * Submits new scores to the score leaderboard. Adds the new scores to the
     * leaderboard sorts and removes duplicates. Does not save, save must happen
     * manually
     * 
     * @param rankings player rankings to update
     */
    public void updateScores(Tuple<Player, Ranking>[] rankings) {
        for (Tuple<Player, Ranking> r : rankings) {
            Highscore oldScore = highscores.get(r.first.getId());
            if (oldScore != null) {
                if (oldScore.getScore() < r.second.points()) {
                    highscores.put(r.first.getId(), new Highscore(r.first.getName(), r.second.points()));
                }
            } else {
                highscores.put(r.first.getId(), new Highscore(r.first.getName(), r.second.points()));
            }
        }
        var list = highscores.entrySet().stream().sorted(sorter).limit(MAX_LENGTH).collect(
                Collectors.toMap(
                        e -> e.getKey(),
                        e -> e.getValue(),
                        (oldValue, newValue) -> newValue,
                        HashMap::new));

        highscores = list;
    }

    /**
     * Gets a sorted list of the current highscore entries to be shown with UI
     *
     * @return a sorted list of highscores
     */
    public Highscore[] getHighscores() {
        return this.highscores.entrySet().stream().sorted(sorter).map((e) -> e.getValue()).toList()
                .toArray(new Highscore[0]);
    }

    /**
     * Data class for the highscores, contains the name and score for the entry
     * 
     * @author Lucas Vanderwielen
     */
    public static class Highscore implements Serializable {
        String username;
        double score;

        public Highscore(String username, double score) {
            this.username = username;
            this.score = score;
        }

        public String getUsername() {
            return this.username;
        }

        public double getScore() {
            return this.score;
        }
    }

    /** Resets the highscores, removals all entries */
    public void reset() {
        this.highscores = new HashMap<>();
    }
}
