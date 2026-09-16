#ifndef TEMPLATES_HPP
#define TEMPLATES_HPP

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

void use_box(Box& b);
void inspect_car_box(Box<Transport::Car>& b);

#endif
