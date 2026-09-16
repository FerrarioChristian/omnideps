#include "namespaces_and_macros.hpp"

namespace MathLib {
    int Calculator::add(int a, int b) {
        return a + b;
    }
}

void use_math() {
    MathLib::Calculator calc;
    calc.add(2, MULTIPLY(3, 4));
}
