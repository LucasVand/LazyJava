
/* Created with LazyJava */

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;

import Web.User;
// import static car.Car.day;

public class Main {

    public static void main(String[] args) throws Exception {
        User u = new User();

        System.out.println("Hello world!");
        System.out.println("Welcome to your LazyJava project");
        // System.out.println("Day: " + day);

        // // 1. Create a client
        // HttpClient client = HttpClient.newHttpClient();
        //
        // // 2. Build a request
        // HttpRequest request = HttpRequest.newBuilder()
        // .uri(URI.create("http://localhost:8080"))
        // .header("Accept", "application/json")
        // .GET()
        // .build();
        //
        // // 3. Send the request and get a response
        // HttpResponse<String> response = client.send(request,
        // HttpResponse.BodyHandlers.ofString());
        //
        // // 4. Print the result
        // System.out.println("Status Code: " + response.statusCode());
        // System.out.println("Body: " + response.body());
    }
}
