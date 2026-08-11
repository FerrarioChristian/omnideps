#ifndef UNTESTED_FEATURES_HPP
#define UNTESTED_FEATURES_HPP

// 1. Template classes
template <typename T>
class Box {
public:
    T value;
    T getValue() { return value; }
};

// 2. Namespaces
namespace MathLib {
    class Calculator {
    public:
        int add(int a, int b);
    };
}

// 3. Macros
#define MULTIPLY(a, b) ((a) * (b))

#endif
