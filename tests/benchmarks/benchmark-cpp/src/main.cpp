#include <iostream>
#include <memory>
#include "Car.h"

int main() {
    std::unique_ptr<Transport::Vehicle> myCar = std::make_unique<Transport::Car>("Toyota", 100, 4);
    
    myCar->accelerate();
    myCar->displayInfo();
    
    return 0;
}
