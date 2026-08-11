#include "untested_features.hpp"

Box::Box(int val) : value(val) {}

int Box::getValue() {
    return value;
}

namespace MathLib {
    int Calculator::add(int a, int b) {
        return a + b;
    }
}

void use_box(Box& b) {
    b.getValue();
}

void use_math() {
    MathLib::Calculator calc;
    calc.add(2, MULTIPLY(3, 4));
}
