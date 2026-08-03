package com.example.math;

import com.example.math.Adder;
import com.example.math.Subtracter;

public class Calc {
    public static int add(int a, int b) {
        return Adder.add(a, b);
    }

    public static int subtract(int a, int b) {
        return Subtracter.subtract(a, b);
    }
}
