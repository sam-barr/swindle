#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

#include "rc.h"

bool is_uninit(RC *rc) {
    return rc->count == NULL && rc->reference == NULL && rc->destructor == NULL;
}

/*
 * Drop this reference, and free memory if this was the last reference
 */
void drop(RC *rc) {
    if(is_uninit(rc)) return;
    *rc->count -= 1;
    if(*rc->count <= 0) {
        rc->destructor(rc->reference);
        free(rc->count);
    }
}

/*
 * This function exists to make generating LLVM easier
 */
void drop2(RC **rc) {
    drop(*rc);
}

/*
 * create a copy of rc
 * actually just increases the count and returns rc, but I think that's fine
 */
RC *alloc(RC *rc) {
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

/*
 * Uninitializes the RC
 */
void uninit(RC *rc) {
    rc->count = NULL;
    rc->reference = NULL;
    rc->destructor = NULL;
}

/*
 * destroy an RC if its count is non-positive
 */
void destroy_noref(RC *rc) {
    if(!is_uninit(rc) && *rc->count <= 0) {
        rc->destructor(rc->reference);
        free(rc->count);
    }
}
