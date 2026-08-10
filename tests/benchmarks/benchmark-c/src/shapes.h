#ifndef SHAPES_H
#define SHAPES_H

typedef union {
    int size;
    float area;
} ShapeData;

struct Rectangle {
    int width;
    int height;
    ShapeData data;
};

typedef struct {
    int radius;
} Circle;

int calculate_area(struct Rectangle* rect);

#endif
