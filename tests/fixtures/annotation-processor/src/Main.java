import processor.MyAnnotation;
import generated.MyGeneratedClass;

@MyAnnotation
public class Main {

    public static void main(String[] args) {
        System.out.println(MyGeneratedClass.getMessage());
    }
}
