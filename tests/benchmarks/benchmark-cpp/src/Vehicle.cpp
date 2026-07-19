#include "Vehicle.h"
#include <iostream>

namespace Transport {
    Vehicle::Vehicle(std::string b, int s) : brand(b), speed(s) {}
    
    Vehicle::~Vehicle() {}

    void Vehicle::displayInfo() {
        std::cout << "Brand: " << brand << ", Speed: " << speed << std::endl;
    }

    std::string Vehicle::getBrand() {
        return brand;
    }
}
