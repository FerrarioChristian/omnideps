#include "Engine.h"
#include <iostream>

namespace automotive {
    V8Engine::V8Engine(int hp) : horsepower(hp) {}

    void V8Engine::start() {
        std::cout << "V8 Engine starting with " << horsepower << " HP!" << std::endl;
    }

    int V8Engine::getHorsepower() {
        return horsepower;
    }
}
