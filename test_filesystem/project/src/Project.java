
/* Created with LazyJava */

import com.fasterxml.jackson.databind.ObjectMapper;

public class Project {

    public static void main(String[] args) {
        System.out.println("Hello world!");
        System.out.println("Welcome to your LazyJava project");

        ObjectMapper mapper = new ObjectMapper();
        String jsonString = "{\"name\":\"John\", \"age\":30}";

        try {
            // Parse into a specific class
            User user = mapper.readValue(jsonString, User.class);
            System.out.println(user.name);
        } catch (Exception e) {
            System.out.println("Error");
            System.out.println(e);
        }

    }

    static class User {
        String name;
        int age;

        public User() {

        }

        public String getName() {
            return name;
        }

        public void setName(String name) {
            this.name = name;
        }

        public int getAge() {
            return age;
        }

        public void setAge(int age) {
            this.age = age;
        }
    }
}
