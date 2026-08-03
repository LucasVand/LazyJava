package saveData;

import java.io.Serializable;

/**
 * Stats, represents an instance of some stats. Contains all info about WPM,
 * peak WPM, accuracy, error count, chars typed and such. Exposed combine
 * methods to aggregate total stats over time and over sessions
 * 
 * @author Lucas Vanderwielen
 */

public class Stats implements Serializable {
    public final static long PEAK_WPM_DURATION = 5 * 1000;
    String accountId;
    long totalPlayTime;
    double avgWPM;
    double peakWPM;
    double accuracy;
    long errorCount;
    long charsTyped;
    long wordsTyped;
    int matchesPlayed;
    int points;
    int highestDifficulty;

    Long wordStartTime;

    History peakWPMHistory;

    public Stats(String accountId) {
        this.accountId = accountId;
        this.peakWPMHistory = new History(Stats.PEAK_WPM_DURATION);

    }

    /**
     * Combines another stats object with this one. This combines values based on
     * meaning. Some are averaged others the max or min is taken
     */
    public void combine(Stats stats) {
        this.totalPlayTime += stats.totalPlayTime;
        this.avgWPM = (this.avgWPM + stats.avgWPM) / 2.0;
        this.peakWPM = Double.max(this.peakWPM, stats.peakWPM);
        this.errorCount += stats.errorCount;
        this.charsTyped += stats.charsTyped;
        this.wordsTyped += stats.wordsTyped;
        this.matchesPlayed += stats.matchesPlayed;
        this.accuracy = 1 - ((double) this.errorCount / (double) this.charsTyped);

        this.points = Integer.max(this.points, stats.points);
        this.highestDifficulty = Integer.max(this.highestDifficulty, stats.highestDifficulty);
    }

    /**
     * Updates all the stats based on the chars given. If the char is incorrect then
     * the error is calculated. If space then wpm stats are calculated
     */
    public void typeChar(char ch, char expected) {
        this.charsTyped += 1;
        if (wordStartTime == null) {
            this.wordStartTime = System.currentTimeMillis();
        }

        if (ch != expected) {

            this.errorCount += 1;
        }
        if (ch == ' ') {
            long duration = System.currentTimeMillis() - wordStartTime;
            double wpm = (60 * 1000 / Long.max(1, duration));
            double newWPM = (this.avgWPM * this.wordsTyped + wpm) / (this.wordsTyped + 1);
            this.avgWPM = newWPM;

            this.wordsTyped += 1;
            this.peakWPMHistory.addEntry();

            double currentPeak = ((double) this.peakWPMHistory.count()) * (60.0 * 1000.0)
                    / (double) Stats.PEAK_WPM_DURATION;

            if (currentPeak > this.peakWPM) {
                this.peakWPM = currentPeak;
            }
            this.wordStartTime = System.currentTimeMillis();
        }
        this.accuracy = 1 - (this.errorCount / this.charsTyped);
    }

    public void addPlayTime(long time) {
        this.totalPlayTime += time;
    }

    /** Increaes the matches played by one */
    public void completeMatch() {
        this.matchesPlayed += 1;
    }

    public void setPoints(int points) {
        this.points = points;
    }

    public void setDifficulty(int dif) {
        this.highestDifficulty = dif;
    }

    public String getAccountId() {
        return accountId;
    }

    public long getTotalPlayTime() {
        return totalPlayTime;
    }

    public double getAvgWPM() {
        return avgWPM;
    }

    public double getPeakWPM() {
        return peakWPM;
    }

    public double getAccuracy() {
        return accuracy;
    }

    public long getErrorCount() {
        return errorCount;
    }

    public long getCharsTyped() {
        return charsTyped;
    }

    public long getWordsTyped() {
        return wordsTyped;
    }

    public int getMatchesPlayed() {
        return matchesPlayed;
    }

    public int getHighestDifficulty() {
        return highestDifficulty;
    }

    public int getHighscore() {
        return this.points;
    }
}
