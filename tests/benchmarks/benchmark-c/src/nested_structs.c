#include "nested_structs.h"

int get_width(BoundingBox box) {
    return box.bottom_right->x - box.top_left->x;
}
