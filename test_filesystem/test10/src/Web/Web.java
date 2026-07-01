
package Web;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;

/**
 * Web
 */
@SpringBootApplication
@RestController
public class Web {

    public static void main(String[] args) {

        SpringApplication.run(Web.class, args);
    }

    @GetMapping("/")
    public String get() {
        return "This is from spring";
    }
}
