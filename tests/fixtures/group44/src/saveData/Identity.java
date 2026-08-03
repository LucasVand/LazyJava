package saveData;

import java.io.Serializable;

/**
 * Identity, represents an account identity. Contains thier username and
 * password and exposes functions to modify and update them. Along with
 * functions to authenticate
 * 
 * @author Lucas Vanderwielen
 */
public class Identity implements Serializable {
    private String username;
    private String password;

    public Identity(String u, String p) {
        this.username = u;
        this.password = p;
    }

    /**
     * Check whether a given username and password matchs the username and password
     * of this identity
     */
    public boolean authenticate(String u, String p) {
        return u.equals(this.username) && p.equals(this.password);
    }

    public String getUsername() {
        return username;
    }

    public String getPassword() {
        return password;
    }

    public void setPassword(String newP) {
        password = newP;
    }
}
