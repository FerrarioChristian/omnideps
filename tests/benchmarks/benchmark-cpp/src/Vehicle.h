#ifndef VEHICLE_H
#define VEHICLE_H

#include <string>

namespace Transport {
    union EngineSpec {
        int horsepower;
        float kw_power;
    };

    class Vehicle {
    protected:
        std::string brand;
        int speed;
        EngineSpec spec;

    public:
        Vehicle(std::string brand, int speed);
        virtual ~Vehicle();
        
        virtual void accelerate() = 0;
        virtual void displayInfo();
        std::string getBrand();
    };
}

#endif
