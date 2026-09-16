#ifndef NAMESPACES_AND_MACROS_HPP
#define NAMESPACES_AND_MACROS_HPP

// 1. Namespaces
namespace MathLib {
    class Calculator {
    public:
        int add(int a, int b);
    };
}

// 2. Macros
#define MULTIPLY(a, b) ((a) * (b))

void use_math();

#endif
