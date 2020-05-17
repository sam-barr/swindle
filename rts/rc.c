#include <stdlib.h>
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
RC *copy(RC *rc) {
    *rc->count += 1;
    return rc;
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
