#include "shapes.h"
#include "math_utils.h"

int calculate_area(Rectangle* rect) {
    return multiply(rect->width, rect->height);
}
