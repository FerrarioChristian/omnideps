#include <stdio.h>
#include "shapes.h"
#include "math_utils.h"

int global_counter = 0;

int main() {
    Rectangle rect;
    rect.width = 10;
    rect.height = 5;

    int area = calculate_area(&rect);
    
    global_counter = add(global_counter, 1);
    
    printf("Area: %d\n", area);
    return 0;
}
