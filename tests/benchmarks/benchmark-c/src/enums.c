#include "enums.h"

int color_to_int(Color c) {
    if (c == RED) {
        return 0;
    } else if (c == GREEN) {
        return 1;
    }
    return 2;
}
