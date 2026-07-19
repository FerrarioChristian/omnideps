#include "Car.h"
#include <iostream>

namespace Transport {
    Car::Car(std::string b, int s, int doors) : Vehicle(b, s), numDoors(doors) {}

    void Car::accelerate() {
        speed += 10;
        std::cout << "Car accelerating. New speed: " << speed << std::endl;
    }

    void Car::displayInfo() {
        Vehicle::displayInfo();
        std::cout << "Doors: " << numDoors << std::endl;
    }
}
