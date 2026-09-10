#include "untested_features.hpp"


namespace MathLib {
    int Calculator::add(int a, int b) {
        return a + b;
    }
}

void use_box(Box& b) {
    b.getValue();
}

void inspect_car_box(Box<Transport::Car>& b) {
    b.getValue().displayInfo();
}

void use_math() {
    MathLib::Calculator calc;
    calc.add(2, MULTIPLY(3, 4));
}
