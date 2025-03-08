#include <string>
#include <iostream>

namespace benchmarks {

class Entity {
public:
    virtual ~Entity() = default;
    virtual std::string getId() const = 0;
};

class LivingBeing : public virtual Entity {
public:
    virtual void breathe() = 0;
};

class Animal : public LivingBeing {
protected:
    std::string name;
public:
    Animal(std::string n) : name(n) {}
    std::string getId() const override { return name; }
    void breathe() override {}
};

class Mammal : public Animal {
public:
    using Animal::Animal;
    virtual void give_birth() {}
};

class Cat : public Mammal {
public:
    using Mammal::Mammal;
    void speak() { std::cout << "Meow\n"; }
    
    // Nested Class
    class Breed {
    public:
        std::string species_type;
        Breed(std::string t) : species_type(t) {}
    };
};

class Robot : public virtual Entity {
public:
    std::string getId() const override { return "Robot-1"; }
    virtual void charge() {}
};

// Ereditarietà Multipla Reale
class Cyborg : public Mammal, public Robot {
public:
    Cyborg(std::string n) : Animal(n), Mammal(n) {}
    std::string getId() const override { return name + "-Cyborg"; }
};

// Nesting profondo e Struct
class Outer {
public:
    struct Inner {
        class DeepInner {
        public:
            void hello() {
                // Local Class
                class LocalProduct {};
            }
        };
    };
};

} // namespace benchmarks
