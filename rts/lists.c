#include <stdlib.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdarg.h>
#include <assert.h>

#include "rc.h"
#include "lists.h"

/*
 * Optimization ideas:
 * For a list of RCs, they (theoretically) should all have the same destructor,
 * so we could save memory be optimizing that out.
 *
 * A list of units doesn't need to actually make the list;
 * just knowing the length is enough.
 */

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
    list->items = malloc(size * count);
    list->capacity = item_type == SW_UNIT ? 0 : count;
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

void index_list(RC *l, int64_t idx, void *out) {
    List *list = (List *)l->reference;
    assert(idx >= 0 && (size_t)idx < list->length);
    switch(list->item_type) {
        case SW_INT: *(int64_t *)out = ((int64_t *)list->items)[idx]; break;
        case SW_BOOL: *(bool *)out = ((bool *)list->items)[idx]; break;
        case SW_UNIT: *(bool *)out = 0;
        case SW_STRING:
        case SW_LIST: *(RC *)out = ((RC *)list->items)[idx]; break;
    }
    destroy_noref(l);
}

int64_t length_list(RC *l) {
    List *list = (List *)l->reference;
    int64_t length = (int64_t)list->length;
    destroy_noref(l);
    return length;
}
