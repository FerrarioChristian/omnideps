#ifndef CAR_H
#define CAR_H

#include "Vehicle.h"

namespace Transport {
    class Car : public Vehicle {
    private:
        int numDoors;

    public:
        Car(std::string brand, int speed, int numDoors);
        
        void accelerate() override;
        void displayInfo() override;
    };
}

#endif
