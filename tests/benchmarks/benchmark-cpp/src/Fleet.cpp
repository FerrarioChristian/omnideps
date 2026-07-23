#include "Fleet.h"

void Fleet::addCar(const Transport::Car& car) {
    cars.push_back(car);
}

int Fleet::getMaxFleetSize() {
    return 100;
}

void Fleet::startAll() {
    for (Transport::Car& car : cars) {
        car.displayInfo();
    }
}
