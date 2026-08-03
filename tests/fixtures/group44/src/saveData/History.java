package saveData;

import java.io.Serializable;
import java.util.ArrayList;
import java.util.List;

/**
 * History can be used to keep track of the number of entries over a time with a
 * time to live. This is created with a ttl (Time To Live) this is how long a
 * given entry will last after it is added. Entries can be added which increases
 * the total entries and they are removed after they expire. This removal
 * happens automatically
 * 
 * @author Lucas Vanderwielen
 */
public class History implements Serializable {
    long ttl;
    List<Long> list;

    /** Create a new history with the specified time to live */
    public History(long ttl) {
        this.list = new ArrayList<>();
        this.ttl = ttl;
    }

    /** Add a new entry increaing the total count */
    public void addEntry() {
        long now = System.currentTimeMillis();
        this.list.add(now);

        this.list.removeIf(e -> {
            return e < now - ttl;
        });

    }

    /** Gets the number of alive entries */
    public int count() {
        return this.list.size();
    }
}
