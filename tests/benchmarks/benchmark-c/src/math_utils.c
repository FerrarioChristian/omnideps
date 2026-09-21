#include "math_utils.h"

int add(int a, int b) {
    return a + b;
}

int multiply(int a, int b) {
    int result = 0;
    for (int i = 0; i < b; i++) {
        result = add(result, a);
    }
    return result;
}

typedef int MyInt;

float divide(int a, int b) {
    return (float)a / (float)b;
}

int cast_typedef_parenthesized(double val) {
    return (MyInt)(val);
}

int cast_typedef_unparenthesized(double val) {
    return (MyInt)val;
}

int call_parenthesized(int a, int b) {
    return (add)(a, b);
}

int square_func(int x) {
    return x * x;
}

int call_single_arg_parenthesized(int a) {
    return (square_func)(a);
}
