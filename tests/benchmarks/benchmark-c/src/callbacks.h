#ifndef CALLBACKS_H
#define CALLBACKS_H

typedef void (*ActionCallback)(int status);

void register_callback(ActionCallback cb);
void trigger_callback();

#endif
