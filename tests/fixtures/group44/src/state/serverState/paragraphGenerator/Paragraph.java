package state.serverState.paragraphGenerator;

/**
 * Paragraph
 * 
 * @author Lucas Vanderwielen
 */
public class Paragraph {
    String paragraph;
    int difficulty;

    public Paragraph(String text, int dif) {
        this.paragraph = text;
        this.difficulty = dif;
    }

    public String getParagraph() {
        return this.paragraph;
    }

    public int getDifficulty() {
        return difficulty;
    }
}
