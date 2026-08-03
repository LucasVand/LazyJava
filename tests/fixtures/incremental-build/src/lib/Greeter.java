package com.example.lib;

import com.example.greeting.Formatter;

public class Greeter {
    public static String greet(String name) {
        return "Hello, " + Formatter.format(name);
    }
}
