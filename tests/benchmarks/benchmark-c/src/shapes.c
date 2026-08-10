#include "shapes.h"
#include "math_utils.h"

int calculate_area(struct Rectangle* rect) {
    return multiply(rect->width, rect->height);
}
