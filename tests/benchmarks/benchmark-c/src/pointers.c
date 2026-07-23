#include "pointers.h"
#include <stdlib.h>

void move_point(Point* p, int dx, int dy) {
    p->x += dx;
    p->y += dy;
}

Point* create_point(int x, int y) {
    Point* p = (Point*)malloc(sizeof(Point));
    p->x = x;
    p->y = y;
    return p;
}
