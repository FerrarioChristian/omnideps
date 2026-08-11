#ifndef UNTESTED_FEATURES_H
#define UNTESTED_FEATURES_H

// 1. Typedef struct with both tag and typedef name
typedef struct Point3D {
    int x;
    int y;
    int z;
} Point3D;

// 2. Anonymous struct with typedef
typedef struct {
    float weight;
} ConfigData;

// 3. Struct declaration only (no typedef), accessed with `struct` keyword
struct NodeData {
    int id;
};

// 4. Complex macro that calls a function and accesses a field
#define LOG_NODE(n) printf("Node ID: %d", n->id)

// 5. Function pointer typedef
typedef int (*ComputeFunc)(int, int);

// 6. Forward declaration of struct without definition
struct OpaqueStruct;
struct OpaqueStruct* get_opaque();

#endif
