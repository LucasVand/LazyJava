package Web;

import javax.annotation.processing.Generated;
import org.jspecify.annotations.Nullable;

@Generated("com.google.auto.value.processor.AutoValueProcessor")
final class AutoValue_Person extends Person {

  private final String name;

  private final int age;

  AutoValue_Person(
      String name,
      int age) {
    if (name == null) {
      throw new NullPointerException("Null name");
    }
    this.name = name;
    this.age = age;
  }

  @Override
  String name() {
    return name;
  }

  @Override
  int age() {
    return age;
  }

  @Override
  public String toString() {
    return "Person{"
        + "name=" + name + ", "
        + "age=" + age
        + "}";
  }

  @Override
  public boolean equals(@Nullable Object o) {
    if (o == this) {
      return true;
    }
    if (o instanceof Person) {
      Person that = (Person) o;
      return this.name.equals(that.name())
          && this.age == that.age();
    }
    return false;
  }

  @Override
  public int hashCode() {
    int h$ = 1;
    h$ *= 1000003;
    h$ ^= name.hashCode();
    h$ *= 1000003;
    h$ ^= age;
    return h$;
  }

}
