#ifndef SHAPES_H
#define SHAPES_H

typedef union {
    int size;
    float area;
} ShapeData;

typedef struct {
    int width;
    int height;
    ShapeData data;
} Rectangle;

int calculate_area(Rectangle* rect);

#endif
