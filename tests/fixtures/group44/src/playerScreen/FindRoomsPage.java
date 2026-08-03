package playerScreen;
import java.awt.AlphaComposite;
import java.awt.BorderLayout;
import java.awt.Dimension;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.GridBagLayout;
import java.awt.image.BufferedImage;
import java.util.ArrayList;

import javax.imageio.ImageIO;
import javax.swing.Box;
import javax.swing.JPanel;
import javax.swing.Timer;

import UIComponents.Flexbox;
import UIComponents.Header;
import UIComponents.PageLayout;
import broadcasting.Room;
import broadcasting.RoomFinder;
import controller.screens.screenEvent.ScreenEventBus;
import utils.ColorManager;

/**
 * Page where users will see open rooms to join,
 * 
 * @author Sam Deitz
 */
public class FindRoomsPage extends JPanel {

    /**
     * Room finder
     */
    private RoomFinder finder;

    /**
     * List of rooms being broadcasted
     */
    private ArrayList<Room> openRooms = new ArrayList<>();

    /**
     * Panel for displaying list of rooms
     */
    private Flexbox roomList;

    /**
     * Background image for the page
     */
    private BufferedImage bgImage;
    
    /**
     * Initialize a new find rooms page. This will display
     * all open rooms and give the option to join them.
     */
    public FindRoomsPage() {

        // Set up page
        setVisible(true);
        setPreferredSize(new Dimension(1200, 700));
        setFocusable(true);
        setLayout(new BorderLayout());
        startSearch();
        getOpenRooms();
        setOpaque(false);

        // Load background
        try {
            bgImage = ImageIO.read(getClass().getResourceAsStream("/resources/images/islandBg.png"));
        } catch (Exception e) {
            System.err.println("Could not load background image!");
            e.printStackTrace();
        }

        // create initial panel for list of rooms
        createRoomList();

        // Create back buttoon for bottom right corner
        PageLayout.createBackButton(this, e -> {
            ScreenEventBus.publish("PLAYER_SCREEN", new PlayerScreenPage());
        });
        
        // update room list every .5 seconds
        Timer t = new Timer(500, e -> {
            renderRoomList();
        });
        t.start();
    }

    /**
     * Create the initial room list panel
     */
    private void createRoomList() {


        // main panel setups
        JPanel wrapper = new JPanel(new GridBagLayout());
        wrapper.setOpaque(false);
        Flexbox mainPanel = new Flexbox(true);
        mainPanel.setPreferredSize(new Dimension(700,500));
        mainPanel.setMinimumSize(new Dimension(700,500));
        mainPanel.setMaximumSize(new Dimension(700,500));
        mainPanel.setOpaque(false);

        // TITLE
        Header title = new Header("All Open Rooms:");
        title.setForeground(ColorManager.primaryBrown);
        title.setAlignmentX(CENTER_ALIGNMENT);

        // LIST OF ROOMS PANEL
        roomList = new Flexbox(true);
        roomList.setAlignmentX(CENTER_ALIGNMENT);

        roomList.setPreferredSize(new Dimension(700,500));
        roomList.setMinimumSize(new Dimension(700,500));
        roomList.setMaximumSize(new Dimension(700,500));

        roomList.setBackground(ColorManager.primarySand);
        roomList.addPadding(20);

        // ADD COMPONENTS
        mainPanel.add(Box.createVerticalStrut(20));
        mainPanel.add(title);
        mainPanel.add(roomList);
        wrapper.add(mainPanel);
        add(wrapper, BorderLayout.CENTER);
    }


    /**
     * Render list of rooms to the room list panel
     */
    private void renderRoomList() {

        // clear panel
        roomList.removeAll();

        // prompt finder for rooms
        getOpenRooms();

        // create and add components for each found room
        for(Room r : openRooms) {
            OpenRoom room = new OpenRoom(r, 5);
            roomList.add(room);
            roomList.add(Box.createVerticalStrut(10)); // Add a gap between rooms
        }

        // justify at top and repaint
        roomList.add(Box.createVerticalGlue());
        roomList.revalidate();
        roomList.repaint();        
    }

    /**
     * Start searching for rooms
     */
    private void startSearch() {
        finder = new RoomFinder();
    }

    /**
     * Get the room list of rooms currently broadcasting
     */
    private void getOpenRooms() {
        openRooms = finder.findRooms();
    }

    /**
     * Draw background
     */
    @Override
    public void paintComponent(Graphics g) {
        super.paintComponent(g);

        if (bgImage != null) {
            Graphics2D g2d = (Graphics2D) g.create();
            float opacity = 0.3f;
            g2d.setComposite(AlphaComposite.getInstance(AlphaComposite.SRC_OVER, opacity));
            g2d.drawImage(bgImage, 0, 0, this.getWidth(), this.getHeight(), null);
            g2d.dispose();
        }
    }

    /**
     * Get the room finder object
     * @return room finder
     */
    public RoomFinder getRoomFinder() {
        return finder;
    }

}
