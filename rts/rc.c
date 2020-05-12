#include <stdlib.h>
#include <stdio.h>
#include <string.h>

typedef void (*Destructor)(void *);

typedef struct RC {
    Destructor destructor;
    void *reference;
    int *count;
} RC;

/*
 * Drop this reference, and free memory if this was the last reference
 */
void drop(RC *rc) {
    *rc->count -= 1;
    if(*rc->count == 0) {
        rc->destructor(rc->reference);
        free(rc->count);
    }
}

/*
 * copy old to new, increasing the number of references by 1
 */
void copy(RC *new, RC *old) {
    *old->count += 1;
    *new = *old;
}

/*
 * Assumes that x has already been malloced
 */
void new(RC *rc, void *x, void (*destructor)(void *)) {
    rc->destructor = destructor;
    rc->reference = x;
    rc->count = malloc(sizeof(int));
    *rc->count = 1;
}

typedef struct OwnsString {
    char *myString;
} OwnsString;

void relinquish(OwnsString *owner) {
    free(owner->myString);
    free(owner);
}

int main() {
    RC rc1;
    new(&rc1, strdup("Hello, World!"), free);
    printf("%s\n", (char *)(rc1.reference));
    RC rc2;
    copy(&rc2, &rc1);
    drop(&rc1);
    printf("%s\n", (char *)(rc2.reference));
    drop(&rc2);

    OwnsString *owner = malloc(sizeof(OwnsString));
    owner->myString = strdup("this is a string");
    new(&rc1, owner, (Destructor) relinquish);
    copy(&rc2, &rc1);
    drop(&rc2);
    drop(&rc1);
}
