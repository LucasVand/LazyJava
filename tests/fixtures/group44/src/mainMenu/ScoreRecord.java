package mainMenu;

import java.awt.Dimension;

import javax.swing.Box;

import UIComponents.Flexbox;
import UIComponents.Subheader;
import utils.ColorManager;

public class ScoreRecord extends Flexbox {

    final int WIDTH = 600;
    final int HEIGHT = 70;

    public ScoreRecord(String placeHeader, String nameHeader, String scoreHeader, boolean header) {
        super();
        addPadding(10);
        setPreferredSize(new Dimension(Integer.MAX_VALUE, HEIGHT));
        setMinimumSize(new Dimension(Integer.MAX_VALUE, HEIGHT));
        setMaximumSize(new Dimension(Integer.MAX_VALUE, HEIGHT));
        setVisible(true);
        if (header) setBackground(ColorManager.primaryBlue);
        else setBackground(ColorManager.secondaryBrown);

        Subheader placeText = new Subheader(placeHeader);
        placeText.setPreferredSize(new Dimension(100, 50));
        placeText.setForeground(ColorManager.primarySand);

        Subheader nameText = new Subheader(nameHeader);
        nameText.setPreferredSize(new Dimension(100, 50));
        nameText.setForeground(ColorManager.primarySand);


        Subheader scoreText = new Subheader(scoreHeader);
        scoreText.setPreferredSize(new Dimension(100, 50));
        scoreText.setForeground(ColorManager.primarySand);
        
        add(placeText);
        add(Box.createHorizontalStrut(15));
        add(nameText);
        add(Box.createHorizontalGlue());
        add(scoreText);
    }
    
}
