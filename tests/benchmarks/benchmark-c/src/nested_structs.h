#ifndef NESTED_STRUCTS_H
#define NESTED_STRUCTS_H

#include "pointers.h"

typedef struct {
    Point* top_left;
    Point* bottom_right;
} BoundingBox;

int get_width(BoundingBox box);

#endif
