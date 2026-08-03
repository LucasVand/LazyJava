package com.example;

import com.example.greeting.Formatter;
import com.example.lib.Greeter;
import com.example.math.Calc;

public class Main {
    public static void main(String[] args) {
        System.out.println(Greeter.greet(Formatter.format("world")));
        System.out.println(Calc.add(1, 2));
        System.out.println(Calc.subtract(5, 3));
    }
}
