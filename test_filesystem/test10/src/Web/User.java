package Web;

import processor.Annot;

/**
 * User
 */
@Annot
public class User {
    String name;
    String password;
    String email;

    public User() {

    }

    public String getName() {
        return this.name;
    }

    public String getPassword() {
        return this.password;
    }
}
