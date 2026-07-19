#ifndef VEHICLE_H
#define VEHICLE_H

#include <string>

namespace Transport {
    class Vehicle {
    protected:
        std::string brand;
        int speed;

    public:
        Vehicle(std::string brand, int speed);
        virtual ~Vehicle();
        
        virtual void accelerate() = 0;
        virtual void displayInfo();
        std::string getBrand();
    };
}

#endif
