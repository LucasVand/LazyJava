import org.json.JSONObject;

/* Created with LazyJava */
public class Project {

    public static void main(String[] args) {
        System.out.println("Hello world!");
        System.out.println("Welcome to your LazyJava project");
        JSONObject jo = new JSONObject("{ \"abc\" : \"def\" }");

        System.out.println(jo.toString());
    }
}
