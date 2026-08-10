#include "math_utils.h"
#include "shapes.h"
#include "pointers.h"
#include "nested_structs.h"
#include "forward_decl.h"
#include "callbacks.h"
#include "globals.h"
#include "enums.h"
#include "macros.h"
#include <stdio.h>

int global_counter = 0;

void my_callback(int status) {
    printf("Callback called with status %d\n", status);
}

int main() {
    struct Rectangle rect = {10, 5};
    int area = calculate_area(&rect);
    
    global_counter = add(global_counter, 1);
    
    Point* p1 = create_point(0, 10);
    Point* p2 = create_point(20, 0);
    move_point(p1, 5, -5);
    
    BoundingBox box;
    box.top_left = p1;
    box.bottom_right = p2;
    
    int w = get_width(box);
    
    Circle c;
    c.radius = 5;
    
    // New test usages
    Node n;
    n.value = 42;
    n.next = NULL;
    process_node(&n);
    
    register_callback(my_callback);
    trigger_callback();
    
    update_state(5);
    int s = system_state;
    
    int c = color_to_int(BLUE);
    
    print_macro_usage();
    
    printf("Area: %d\n", area);
    printf("Width: %d\n", w);
    return 0;
}
