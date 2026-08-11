#include "advanced.h"

ProcessState current_state = STATE_IDLE;
EventHandler active_handler = 0;

void register_handler(EventHandler handler) {
    active_handler = handler;
    current_state = STATE_RUNNING;
}

void fire_event() {
    if (active_handler != 0) {
        active_handler(42);
    }
}
