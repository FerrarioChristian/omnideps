#include <stdio.h>
#include <stdlib.h>

struct Entity {
    char *id;
    char* (*get_id)(struct Entity *self);
};

struct LivingBeing {
    struct Entity base;
    void (*breathe)(struct LivingBeing *self);
};

struct Animal {
    struct LivingBeing base;
    char *name;
};

struct Cat {
    struct Animal base;
    void (*speak)(struct Cat *self);
    
    // Nested struct definition
    struct Breed {
        char *species_type;
    } breed;
};

// Nesting profondo di struct
struct Outer {
    struct Inner {
        struct DeepInner {
            int value;
        } deep;
    } inner;
};

void factory() {
    // Local struct
    struct LocalProduct {
        int id;
    };
    struct LocalProduct p = {1};
}

int main() {
    return 0;
}
