
package Web;

import com.google.auto.value.AutoValue;

@AutoValue
abstract class Person {
    abstract String name();

    abstract int age();

    static Person create(String name, int age) {
        return new AutoValue_Person(name, age);
    }
}
