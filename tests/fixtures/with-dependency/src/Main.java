import org.apache.commons.lang3.StringUtils;

public class Main {

    public static void main(String[] args) {
        String message = "hello world from lazy-java";
        String capitalized = StringUtils.capitalize(message);
        System.out.println(capitalized);

        String abbr = StringUtils.abbreviate(message, 12);
        System.out.println(abbr);

        String reversed = StringUtils.reverse(message);
        System.out.println(reversed);
    }
}
