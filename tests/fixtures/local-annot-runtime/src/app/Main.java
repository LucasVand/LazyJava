package app;

import runtime.MyAnno;
import runtime.RuntimeHelper;
import generated.GeneratedHello;

@MyAnno
public class Main {

    public static void main(String[] args) {
        System.out.println(RuntimeHelper.greet());
        System.out.println(GeneratedHello.msg());
    }
}