package app;

public class Main {

    public static void main(String[] args) {
        Person p = Person.create("Alice", 30);
        System.out.println(p.name());
        System.out.println(p.age());
    }
}
