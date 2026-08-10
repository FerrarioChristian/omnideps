#include "callbacks.h"
#include <stddef.h>

static ActionCallback active_callback = NULL;

void register_callback(ActionCallback cb) {
    active_callback = cb;
}

void trigger_callback() {
    if (active_callback != NULL) {
        active_callback(1);
    }
}
