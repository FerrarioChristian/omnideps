#include "Car.h"
#include "Engine.h"
#include "Fleet.h"
#include <iostream>

using namespace Transport;
using namespace automotive;

int main() {
    Car myCar("Toyota", 120, 4);
    myCar.accelerate();
    myCar.displayInfo();
    
    V8Engine engine(450);
    engine.start();
    
    Fleet myFleet;
    myFleet.addCar(myCar);
    myFleet.startAll();
    
    std::cout << "Max fleet size: " << Fleet::getMaxFleetSize() << std::endl;

    return 0;
}

using MyEngine = automotive::IEngine;

void do_cast() {
    float f = (float)10;
}
