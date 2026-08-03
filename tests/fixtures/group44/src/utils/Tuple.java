package utils;

/**
 * Tuple
 * 
 * @author Lucas Vanderwielen
 */
public class Tuple<T, V> {
  public T first;
  public V second;

  public Tuple(T first, V second) {
    this.first = first;
    this.second = second;
  }

}
