#ifndef UNTESTED_FEATURES_HPP
#define UNTESTED_FEATURES_HPP

#include "Car.h"
#include <vector>

// 1. Template classes
template <typename T>
class Box {
public:
    T value;
    T getValue() { return value; }
};

// 1.1 Multiple template parameters
template <typename K, typename V>
class KeyValue {
public:
    K key;
    V value;
};

// 1.2 Concrete generic usage & nested generics
class CarStorage {
public:
    Box<Transport::Car> single_car;
    std::vector<Box<Transport::Car>> history;
    KeyValue<int, Transport::Car> indexed_car;
};

void inspect_car_box(Box<Transport::Car>& b);

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
