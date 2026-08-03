package playerScreen;

import java.awt.BorderLayout;
import java.awt.Component;
import java.awt.Dimension;
import java.awt.GridLayout;

import javax.swing.BorderFactory;
import javax.swing.Box;
import javax.swing.BoxLayout;
import javax.swing.JPanel;

import UIComponents.Header;
import UIComponents.Subheader;
import saveData.Account;
import saveData.Stats;
import utils.ColorManager;
import utils.FontManager;

class PlayerStatsCard extends JPanel {

    public PlayerStatsCard(Account account) {
        Stats stats = account.getStats();
        setPreferredSize(new Dimension(410, 430));
        setLayout(new BorderLayout());
        setBackground(ColorManager.primarySand);

        JPanel content = new JPanel();
        content.setOpaque(false);
        content.setBorder(BorderFactory.createEmptyBorder(18, 26, 18, 26));
        content.setLayout(new BoxLayout(content, BoxLayout.Y_AXIS));

        Header cardTitle = new Header("Your Statistics");
        cardTitle.setFont(FontManager.getFont(44));
        cardTitle.setForeground(ColorManager.primaryBrown);
        cardTitle.setAlignmentX(Component.CENTER_ALIGNMENT);

        content.add(cardTitle);
        content.add(Box.createRigidArea(new Dimension(0, 18)));

        content.add(createStatsRow(String.format("%.2f WPM", stats.getAvgWPM()), "Average WPM",
                String.format("%.2f", stats.getPeakWPM()) + " WPM", "Peak WPM"));
        content.add(Box.createRigidArea(new Dimension(0, 12)));
        content.add(
                createStatsRow(String.format("%.2f%%", stats.getAccuracy() * 100.0), "Accuracy",
                        stats.getErrorCount() + "", "Errors"));
        content.add(Box.createRigidArea(new Dimension(0, 12)));
        double playTime = (double) stats.getTotalPlayTime() / (double) (1000 * 60);
        content.add(
                createStatsRow(String.format("%.2f mins", playTime), "Total Time Played", stats.getHighscore() + " points",
                        "High Score"));
        content.add(Box.createRigidArea(new Dimension(0, 12)));
        content.add(createStatsRow(stats.getHighestDifficulty() + "", "Highest Level Reached",
                stats.getWordsTyped() + "", "Words Typed"));

        add(content, BorderLayout.CENTER);
    }

    private JPanel createStatsRow(String leftValue, String leftLabel, String rightValue, String rightLabel) {
        JPanel row = new JPanel(new GridLayout(1, 2, 22, 0));
        row.setOpaque(false);
        row.add(createStatCell(leftValue, leftLabel));
        row.add(createStatCell(rightValue, rightLabel));
        return row;
    }

    private JPanel createStatCell(String value, String label) {
        JPanel cell = new JPanel();
        cell.setOpaque(false);
        cell.setLayout(new BoxLayout(cell, BoxLayout.Y_AXIS));

        Header valueText = new Header(value);
        valueText.setFont(FontManager.getFont(28));
        valueText.setForeground(ColorManager.primaryBlue);
        valueText.setAlignmentX(Component.LEFT_ALIGNMENT);

        Subheader labelText = new Subheader(label);
        labelText.setFont(FontManager.getFont(14));
        labelText.setForeground(ColorManager.secondaryBrown);
        labelText.setAlignmentX(Component.LEFT_ALIGNMENT);

        cell.add(valueText);
        cell.add(labelText);
        return cell;
    }
}
