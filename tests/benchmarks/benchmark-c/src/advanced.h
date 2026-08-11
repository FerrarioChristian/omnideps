#ifndef ADVANCED_H
#define ADVANCED_H

typedef enum {
    STATE_IDLE,
    STATE_RUNNING
} ProcessState;

typedef void (*EventHandler)(int event_id);

extern ProcessState current_state;

void register_handler(EventHandler handler);

#endif
