package eventSystem.events;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.ObjectInputStream;
import java.io.ObjectOutputStream;

import eventSystem.networking.RawEvent;

/**
 * A class for encoding an decoding events into {@code RawEvent} for the
 * networking classes to interact with, internally used inside of
 * {@code ClientEventEngine} and {@code ServerEventEngine}
 * etc
 * 
 * @author Lucas Vanderwielen
 */
public class EventCoder {
    public static RawEvent encodeEvent(Event event) throws IOException {

        ByteArrayOutputStream stream = new ByteArrayOutputStream();
        ObjectOutputStream outputStream = new ObjectOutputStream(stream);

        outputStream.writeObject(event);
        outputStream.flush();

        byte[] bytes = stream.toByteArray();

        return new RawEvent(bytes);
    }

    public static Event decodeEvent(RawEvent rawEvent) throws IOException {
        ByteArrayInputStream byteStream = new ByteArrayInputStream(rawEvent.msg());
        ObjectInputStream inputStream = new ObjectInputStream(byteStream);

        try {
            Event event = (Event) inputStream.readObject();
            return event;
        } catch (ClassNotFoundException e) {
            throw new IOException("Class not found");
        }

    }
}
