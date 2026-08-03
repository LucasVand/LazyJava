package playerScreen;

import java.awt.GridBagLayout;

import javax.swing.JPanel;

import saveData.Account;

public class PlayerPlaceholder extends JPanel {

    public PlayerPlaceholder(Account account) {
        setOpaque(false);
        setLayout(new GridBagLayout());
        add(new PlayerStatsCard(account));
    }
}
