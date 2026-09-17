#include "templates.hpp"

void use_box(Box& b) {
    b.getValue();
}

void inspect_car_box(Box<Transport::Car>& b) {
    b.getValue().displayInfo();
}
