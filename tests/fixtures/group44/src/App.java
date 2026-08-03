import javax.swing.JFrame;

import controller.screens.ScreenController;
import login.LoginPage;
import mainMenu.InstructionsPage;
import mainMenu.MainMenuPage;
import mainMenu.ViewHighScoresPage;
import parentalControls.CreateAccountPage;
import parentalControls.ParentTeacherLoginPage;
import parentalControls.TeacherControlsPage;

/**
 * Main class of the game. Creates JFrame and all static pages. 
 * Initialize the screen controller and starts the UI flow.
 * 
 * @author Sam Deitz
 */
public class App {

    /**
     * Main program, initializes JFrame and starts controller
     */
    public static void main(String[] args) {
        // Initialize JFrame
        JFrame mainFrame = new JFrame("Party Islands");
        mainFrame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
        mainFrame.setVisible(true);
        mainFrame.setResizable(false);

        // Initialize controller
        ScreenController gameSession = new ScreenController(mainFrame);

        // Initialize static pages
        MainMenuPage mp = new MainMenuPage();
        InstructionsPage in = new InstructionsPage();
        ViewHighScoresPage hs = new ViewHighScoresPage();
        LoginPage lp = new LoginPage();
        ParentTeacherLoginPage ptl = new ParentTeacherLoginPage();
        TeacherControlsPage tc = new TeacherControlsPage();
        CreateAccountPage ca = new CreateAccountPage();

        // register static pages with ScreenController
        gameSession.registerScreen("MAIN_MENU", mp);
        gameSession.registerScreen("LOGIN", lp);
        gameSession.registerScreen("HIGH_SCORES", hs);
        gameSession.registerScreen("INSTRUCTIONS", in);
        gameSession.registerScreen("ADMIN_LOGIN", ptl);
        gameSession.registerScreen("ADMIN_CONTROLS", tc);
        gameSession.registerScreen("CREATE_ACCOUNT", ca);

        // Show starting screen and start the UI flow
        gameSession.showScreen("MAIN_MENU");
        gameSession.start();
    }
}
