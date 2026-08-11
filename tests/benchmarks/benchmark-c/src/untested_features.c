#include "untested_features.h"
#include <stdio.h>

void use_point3d(Point3D* p) {
    p->x = 10;
}

void use_config(ConfigData* c) {
    c->weight = 1.0f;
}

void use_node_data(struct NodeData* n) {
    n->id = 5;
    LOG_NODE(n);
}

int execute_compute(ComputeFunc func, int a, int b) {
    return func(a, b);
}
