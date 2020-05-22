#include <stdlib.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdarg.h>
#include <assert.h>

#include "rc.h"
#include "lists.h"

#define GROW_CAPACITY(capacity) (2 * (capacity))
#define MIN_CAPACITY 8
#define MAX(A,B) ((A) > (B) ? (A) : (B))

void destroy_list(List *list) {
    switch(list->item_type) {
        case SW_STRING:
        case SW_LIST:
            for(size_t i = 0; i < list->length; i++)
                drop(((RC *)list->items) + i);
            break;
        default: break;
    }
    free(list->items);
    free(list);
}

size_t item_size(ItemType item_type) {
    switch(item_type){
        case SW_INT: return sizeof(int64_t);
        case SW_BOOL: return sizeof(int);
        case SW_UNIT: return 0;
        case SW_STRING:
        case SW_LIST: return sizeof(RC);
    }
}

void rc_list(RC *rc, ItemType item_type, size_t count, ...) {
    size_t size = item_size(item_type);
    List *list = malloc(sizeof(List));
    list->capacity = item_type == SW_UNIT ? 0 : MAX(count, MIN_CAPACITY);
    list->items = malloc(size * list->capacity);
    list->length = count;
    list->item_type = item_type;

    va_list ap;
    va_start(ap, count);
    for(size_t i = 0; i < count; i++) {
        switch(item_type) {
            case SW_INT:
                ((int64_t *)list->items)[i] = va_arg(ap, int64_t);
                break;
            case SW_BOOL:
                // apparently its better to do int instead of bool
                ((bool *)list->items)[i] = va_arg(ap, int);
                break;
            case SW_UNIT:
                break;
            case SW_STRING:
            case SW_LIST:
                ((RC *)list->items)[i] = *alloc(va_arg(ap, RC *));
                break;
        }
    }
    va_end(ap);

    new(rc, list, (Destructor) destroy_list);
}

ListItem index_list(RC *l, int64_t idx) {
    List *list = (List *)l->reference;
    assert(idx >= 0 && (size_t)idx < list->length);

    ListItem item;
    switch(list->item_type) {
        case SW_INT: item.n = ((int64_t *)list->items)[idx]; break;
        case SW_BOOL: item.b = ((bool *)list->items)[idx]; break;
        case SW_UNIT: item.u = 0;
        case SW_STRING:
        case SW_LIST: item.rc = ((RC *)list->items) + idx; break;
    }
    destroy_noref(l);

    return item;
}

int64_t length_list(RC *l) {
    List *list = (List *)l->reference;
    int64_t length = (int64_t)list->length;
    destroy_noref(l);
    return length;
}

int64_t as_int(ListItem item) {
    return item.n;
}

bool as_bool(ListItem item) {
    return item.b;
}

bool as_unit(ListItem item) {
    return item.u;
}

RC *as_rc(ListItem item) {
    return item.rc;
}

// varargs is a hack to accept any type as input
void push_(RC *l, ...) {
    List *list = (List *)l->reference;

    if(list->length == list->capacity) {
        list->capacity = GROW_CAPACITY(list->capacity);
        list->items = realloc(list->items, item_size(list->item_type) * list->capacity);
    }
    //printf("%zu   %zu\n", list->capacity, list->length);

    va_list ap;
    va_start(ap, l);
    switch(list->item_type) {
        case SW_INT:
            ((int64_t *)list->items)[list->length] = va_arg(ap, int64_t);
            break;
        case SW_BOOL:
            ((bool *)list->items)[list->length] = va_arg(ap, int);
            break;
        case SW_UNIT: // unit list just keeps track of the length
            break;
        case SW_STRING:
        case SW_LIST:
            ((RC *)list->items)[list->length] = *alloc(va_arg(ap, RC *));
            break;
    }
    va_end(ap);
    list->length += 1;

    // NOTE: do NOT destroy_noref here, since no reference is had
    // while a While loop is building a list
}
