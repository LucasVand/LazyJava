package eventSystem.events;

import java.io.Serializable;
import java.util.UUID;

/**
 * This is the base event class, this should not be extended directly use
 * {@code ResponseEvent} or {@code StandaloneEvent} for creating new event types
 *
 * @author Lucas Vanderwielen
 * 
 * 
 */
public class Event implements Serializable {
    String id;
    long time;

    public Event() {
        id = UUID.randomUUID().toString();
        time = System.currentTimeMillis();
    }

    public String eventId() {
        return id;
    }

    public long createTime() {
        return time;
    }

    public void setEventId(String id) {
        this.id = id;
    }
}
