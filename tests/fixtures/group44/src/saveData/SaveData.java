package saveData;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.ObjectInputStream;
import java.io.ObjectOutputStream;
import java.io.Serializable;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;

/**
 * SaveData, main object for managing data that needs to be persisant. Contains
 * all info about accounts and highscores. Will create new if any errors occur
 * while reading the save file
 * 
 * @author Lucas Vanderwielen
 */
public class SaveData implements Serializable {
    static final String SAVE_LOCATION = "./data.bin";
    static final String ADMIN_PASSWORD = "123456";

    private ArrayList<Account> accounts;

    Account currentAccount;
    HighscoreTable highscoreTable;

    static SaveData data;

    public boolean allowSaving = true;

    SaveData() {
        this.accounts = new ArrayList<>();
        this.highscoreTable = new HighscoreTable();

        // remove this later
        accounts.add(new Account("Lucas", "1234"));
        accounts.add(new Account("Sam", "1234"));
        accounts.add(new Account("Sanad", "1234"));
        accounts.add(new Account("Ali", "1234"));
        accounts.add(new Account("Arielle", "1234"));

    }

    // this is the static block that gets run to load the file or create a new save
    // data if loading fails
    static {
        Path path = Paths.get(SaveData.SAVE_LOCATION);

        boolean exists = Files.exists(path);
        if (exists) {
            System.out.println("Loading from save file");
            try {
                byte[] bytes = Files.readAllBytes(path);
                loadFromBytes(bytes);
            } catch (IOException e) {
                System.out.println("Error loading save file");
                e.printStackTrace();
            }
        } else {
            System.out.println("Loading from new");
            loadFromNew();
        }
    }

    /**
     * Adds an account to the save data. This does not auto save and {@code save()}
     * must be manually called
     *
     */
    public void createAccount(Account a) {
        this.accounts.add(a);
    }

    /**
     * Gets the currently logged in account. This will throw a runtime exception if
     * no account is logged in
     */
    public Account getLoggedInAccount() {
        if (currentAccount == null) {
            throw new RuntimeException("No account logged in, to access the current account you must log in");
        }
        return this.currentAccount;
    }

    /**
     * Gets the save data. This will never return null, it is always populated
     */
    static public SaveData getData() {
        return SaveData.data;
    }

    /**
     * Logs an account with the given username and password in and returns the
     * status of the login. If an account with that username and password was found
     * it will be set to the current logged in account and will return true. If no
     * account exists then it will return false and current logged in account will
     * remain unchanged
     */
    public boolean logIn(String username, String password) {
        for (Account a : accounts) {
            if (a.identity.authenticate(username, password)) {
                logInAccount(a);
                return true;
            }
        }
        return false;
    }

    /**
     * Sets the current logged in account to null, which effectively logs the
     * account out
     */
    public void logoutAccount() {
        this.currentAccount = null;
    }

    public HighscoreTable getHighscoreTable() {
        return this.highscoreTable;
    }

    void logInAccount(Account a) {
        this.currentAccount = a;
    }

    /**
     * checks where the given password is equal to the admin password. This has no
     * effect on the current logged in account and should only be used to block
     * admin controls
     */
    public boolean adminLogin(String password) {
        return password.equals(SaveData.ADMIN_PASSWORD);
    }

    /**
     * Takes the current data and saves it to a file. Saving must all be done
     * manually, no saving happens automatically
     */
    public static void save() {
        if (!data.allowSaving) {
            return;
        }

        Path path = Paths.get(SaveData.SAVE_LOCATION);

        try {
            ByteArrayOutputStream stream = new ByteArrayOutputStream();
            ObjectOutputStream outputStream = new ObjectOutputStream(stream);

            outputStream.writeObject(data);
            outputStream.flush();

            byte[] bytes = stream.toByteArray();
            Files.write(path, bytes);
        } catch (IOException e) {
            System.out.println("Unable to save, Error " + e);

        }

    }

    static void loadFromNew() {
        SaveData.data = new SaveData();
        SaveData.save();
    }

    /**
     * Gets a save data from new. This function should not be used and is only
     * exposed for testing
     */
    public static SaveData getFromNew() {
        SaveData.data = new SaveData();
        return data;
    }

    static void loadFromBytes(byte[] bytes) {
        try {
            ByteArrayInputStream byteStream = new ByteArrayInputStream(bytes);
            ObjectInputStream inputStream = new ObjectInputStream(byteStream);

            try {
                SaveData data = (SaveData) inputStream.readObject();
                SaveData.data = data;

            } catch (ClassNotFoundException e) {
                System.out.println("Class not found");
            }
        } catch (IOException e) {
            System.out.println("Unabel to decode byte stream for save file");

            loadFromNew();
        }
    }

    /**
     * Removes the account given from the save data. If the account does not exist
     * nothing happens. Save must be called to save the data
     */
    public void deleteAccount(Account acc) {
        for (int i = 0; i < accounts.size(); i++) {
            // Checks if it's the exact account we want to delete
            if (accounts.get(i).equals(acc)) {
                accounts.remove(i);
                break; // Stops the loop immediately so the list doesn't shift and crash
            }
        }
    }

    /**
     * Gets all the account accociated with the save data object
     */
    public ArrayList<Account> getAccounts() {
        return accounts;
    }

}
