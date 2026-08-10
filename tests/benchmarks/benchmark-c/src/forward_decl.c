#include "forward_decl.h"
#include <stdio.h>

struct Node {
    int value;
    Node* next;
};

void process_node(Node* n) {
    if (n != NULL) {
        printf("Node value: %d\n", n->value);
    }
}
