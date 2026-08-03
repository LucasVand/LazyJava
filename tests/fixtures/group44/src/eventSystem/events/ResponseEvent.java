package eventSystem.events;

/**
 * An event type which recieves a response. This event can be though of as a
 * Question, we send something and then we expect a response back. This should
 * be extended by other classes to create new events
 *
 * @param <T> the response type for this event, the event will be sent and this
 *            is the type of the response
 * @author Lucas Vanderwielen
 */
public class ResponseEvent<T extends StandaloneEvent> extends Event {

}
