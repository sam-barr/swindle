#include <stdlib.h>
#include <string.h>

#include "rc.h"

/*
 * Drop this reference, and free memory if this was the last reference
 */
void drop(RC *rc) {
    *rc->count -= 1;
    if(*rc->count <= 0) {
        rc->destructor(rc->reference);
        free(rc->count);
    }
}

/*
 * create a copy of rc
 * actually just increases the count and returns rc, but I think that's fine
 */
void alloc(RC *rc) {
    *rc->count += 1;
}

/*
 * Assumes that x has already been malloced
 * NOTE: reference count starts as 0, caller should manually increment if needed
 */
void new(RC *rc, void *reference, Destructor destructor) {
    rc->destructor = destructor;
    rc->reference = reference;
    rc->count = malloc(sizeof(int));
    *rc->count = 0;
}

void rc_string(RC *rc, char *s) {
    char *allocated = strdup(s);
    new(rc, allocated, free);
}
