import java.io.IOException;
import java.io.InputStream;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;

/**
 * Main
 */
public class Main {

    public static void main(String[] args) throws IOException, InterruptedException {

        // Load the file as a stream
        try (InputStream is = Main.class.getResourceAsStream("text.txt")) {
            if (is == null) {
                System.out.println("Error: File not found in resources!");
                return;
            }

            // Directly pipe the file stream into the console print stream
            is.transferTo(System.out);

        } catch (Exception e) {
            e.printStackTrace();
        }

        // 1. Create a client
        HttpClient client = HttpClient.newHttpClient();

        // 2. Build a request
        HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create("http://localhost:8080"))
                .header("Accept", "application/json")
                .GET()
                .build();

        // 3. Send the request and get a response
        HttpResponse<String> response = client.send(request, HttpResponse.BodyHandlers.ofString());

        // 4. Print the result
        System.out.println("Status Code: " + response.statusCode());
        System.out.println("Body: " + response.body());
    }
}
