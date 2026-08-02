import java.io.BufferedReader;
import java.io.InputStream;
import java.io.InputStreamReader;

public class Main {
    public static void main(String[] args) throws Exception {
        InputStream stream = Main.class.getResourceAsStream("/hello.txt");
        if (stream == null) {
            System.out.println("Resource hello.txt was not found on the classpath!");
            return;
        }
        BufferedReader reader = new BufferedReader(new InputStreamReader(stream));
        String content = reader.readLine();
        System.out.println("Resource content: " + content);
    }
}
