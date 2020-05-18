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
void drop(RC **rc) {
    RC *rc2 = *rc;
    if(is_uninit(rc2)) {
        return;
    }
    *rc2->count -= 1;
    if(*rc2->count <= 0) {
        rc2->destructor(rc2->reference);
        free(rc2->count);
    }
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
 * Creates a RC for a string (usually a string constant)
 * creates a new, malloced copy of s
 */
void rc_string(RC *rc, char *s) {
    char *allocated = strdup(s);
    new(rc, allocated, free);
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
 * Frees the memory if no references are held
 */
void destruct_if0(RC *rc) {
    if(*rc->count <= 0) {
        rc->destructor(rc->reference);
        free(rc->count);
    }
}
