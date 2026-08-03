package saveData;

import java.io.Serializable;
import java.util.UUID;

/**
 * Account, represents a users account. Contains all information about a user.
 * Contains identity info, stats info, along with an Id. This is saved inside of
 * the {@code SaveData} object and user accounts can be accessed from there
 * 
 * @author Lucas Vanderwielen
 */
public class Account implements Serializable {
    Identity identity;
    String id;
    Stats stats;

    /**
     * Creates a new Account with the specified username and password 
     * 
     * @param u username
     * @param p password
     */
    public Account(String u, String p) {
        this.identity = new Identity(u, p);
        this.id = UUID.randomUUID().toString();
        this.stats = new Stats(this.id);
    }

    public Identity getIdentity() {
        return identity;
    }

    public String getId() {
        return id;
    }

    public Stats getStats() {
        return stats;
    }

    /**
     * Resets the stats of the given account. This cannot be undone and it must be
     * saved after calling this function. Does not save automatically
     */
    public void resetStats() {
        this.stats = new Stats(this.id);
    }
}
