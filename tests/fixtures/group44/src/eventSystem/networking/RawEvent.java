package eventSystem.networking;

import java.nio.ByteBuffer;

public record RawEvent(byte[] msg) {
  public static final int MAX_BYTEBUFFER = 4096;

  public static ByteBuffer encode(RawEvent event) {
    // get the lenght of the message
    int length = event.msg().length;
    // allocate a buffer of the correct length
    ByteBuffer buf = ByteBuffer.allocate(4 + length);

    // put the length into the buffer
    buf.putInt(length);

    // put the message into the buffer
    buf.put(event.msg());
    // ?? im flipping it into read mode ?? maybe get rid
    buf.flip();

    return buf;
  }

  public static RawEvent decode(ByteBuffer buf) {

    // if we have less then 4 bytes then do nothing
    if (buf.remaining() < 4)
      return null;

    // remember the spot we started
    buf.mark();
    // gets the content length
    int length = buf.getInt();

    // if the length will overflow or underflow our buffer throw error
    if (length > MAX_BYTEBUFFER || length < 0) {
      // this is bad we should have some exception here
      throw new RuntimeException("Length is larger then buffer");
    }

    if (buf.remaining() < length) {
      // resets the bufer postition to the spot we remembered because we dont have
      // enough data yet
      buf.reset();
      return null;
    }

    // create the new array and rawevent
    byte[] msg = new byte[length];

    buf.get(msg);
    return new RawEvent(msg);
  }
}
