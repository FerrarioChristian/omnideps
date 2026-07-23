#ifndef FLEET_H
#define FLEET_H

#include "Car.h"
#include <vector>

class Fleet {
private:
    std::vector<Transport::Car> cars;
public:
    void addCar(const Transport::Car& car);
    static int getMaxFleetSize();
    void startAll();
};

#endif
