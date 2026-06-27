package first;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RestController;

@SpringBootApplication
@RestController
public class DemoApplication {

    public static void main(String[] args) {
        SpringApplication.run(DemoApplication.class, args);

    }

    @GetMapping("/")
    public String hello() {
        System.out.println("Hello");
        return "Hello from Spring Boot!";
    }

    @PostMapping("/")
    public String post() {
        System.out.println("Post found");

        return "Posted sucess";
    }
}
