package UIComponents;

import java.awt.BorderLayout;
import java.awt.Color;
import java.awt.Dimension;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.RenderingHints;

import javax.swing.BorderFactory;
import javax.swing.JLabel;
import javax.swing.JPanel;
import javax.swing.JPasswordField;
import javax.swing.JTextField;
import javax.swing.text.JTextComponent;

import utils.ColorManager;
import utils.FontManager;

/**
 * This class represents an input field (password or basic) for a form.
 * 
 * @author Sam Deitz
 */
public class InputField extends JPanel {

    public enum Type {
        BASIC,
        PASSWORD
    }
    
    private JTextComponent field;

    /**
     * Create an input field of a specified type at size 10
     * @param type BASIC or PASSWORD
     * @param label what the text box is for
     * 
     * {@snippet : 
     * InputField i = new InputField(InputField.Type.BASIC, "Username:");
     * add(i);
     * }
     */
    public InputField(Type type, String label) {
        this(10, type, label);
    }

    /**
     * Create an input field of a specified type at any size
     * @param size size of input field
     * @param type BASIC or PASSWORD
     * 
     * {@snippet :
     * InputField i = new InputField(20, InputField.Type.PASSWORD, "Password:");
     * add(i);
     * }
     */
    public InputField(int size, Type type, String label) {
        setOpaque(false);
        setLayout(new BorderLayout(10, 0));
        
        // Label
        JLabel fieldLabel = new JLabel(label);

        // Text field
        if (type == Type.PASSWORD) {
            field = new JPasswordField(size);
        } else field = new JTextField(size);


        // remove default styling
        field.setBorder(null);
        field.setOpaque(false);
        field.setFont(FontManager.getFont(22));
        field.setForeground(ColorManager.secondaryBrown);
        

        // Font and padding
        fieldLabel.setFont(FontManager.getFont(24f));
        fieldLabel.setForeground(ColorManager.primaryBrown);
        setBorder(BorderFactory.createEmptyBorder(10,12,10,12));

        // add to panel
        add(fieldLabel, BorderLayout.WEST);
        add(field, BorderLayout.CENTER);
    }

    @Override
    protected void paintComponent(Graphics g) {
        Graphics2D g2 = (Graphics2D) g.create();
        g2.setRenderingHint(RenderingHints.KEY_ANTIALIASING,
                            RenderingHints.VALUE_ANTIALIAS_ON);

        g2.setColor(Color.white);
        g2.fillRect(0, 0, getWidth(), getHeight());
        super.paintComponent(g);
    }

    /**
     * Get the input in this component
     * @return string containing input
     */
    public String getText() {

        // if it is a password field
        if (field instanceof JPasswordField jPasswordField) {
            String passw = "";
            for(char c : jPasswordField.getPassword()) {
                passw += c;
            }
            return passw;
        }

        // if it is a text field
        return field.getText();
    }

    /**
     * Set the text in the input box
     * @param text new text
     */
    public void setText(String text) {
        field.setText(text);
    }

    @Override
    public Dimension getMaximumSize() {
        // Let it stretch horizontally to infinity, but lock the height
        return new Dimension(Integer.MAX_VALUE, getPreferredSize().height); 
    }

}
