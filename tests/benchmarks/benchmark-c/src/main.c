#include "math_utils.h"
#include "shapes.h"
#include "pointers.h"
#include "nested_structs.h"
#include <stdio.h>

int global_counter = 0;

int main() {
    Rectangle rect = {10, 5};
    int area = calculate_area(&rect);
    
    global_counter = add(global_counter, 1);
    
    Point* p1 = create_point(0, 10);
    Point* p2 = create_point(20, 0);
    move_point(p1, 5, -5);
    
    BoundingBox box;
    box.top_left = p1;
    box.bottom_right = p2;
    
    int w = get_width(box);
    
    printf("Area: %d\n", area);
    printf("Width: %d\n", w);
    return 0;
}
