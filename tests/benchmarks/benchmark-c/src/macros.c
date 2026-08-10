#include "macros.h"
#include <stdio.h>

void print_macro_usage() {
    int buf[MAX_BUFFER];
    int area = SQUARE(5);
    printf("Max buffer is %d, area is %d\n", MAX_BUFFER, area);
}
