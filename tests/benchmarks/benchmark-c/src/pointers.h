#ifndef POINTERS_H
#define POINTERS_H

typedef struct {
    int x;
    int y;
} Point;

void move_point(Point* p, int dx, int dy);
Point* create_point(int x, int y);

#endif
