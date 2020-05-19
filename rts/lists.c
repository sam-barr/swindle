#include <stdlib.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdarg.h>

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
    if(list->item_type == SW_RC) {
        for(size_t i = 0; i < list->length; i++) {
            RC rc = list->items[i].sw_rc;
            drop(&rc);
        }
    }
    free(list->items);
}

void new_list(List *list, ItemType item_type, size_t count, ...) {
    list->items = malloc(sizeof(ListItem) * count);
    list->length = list->capacity = count;
    list->item_type = item_type;

    va_list ap;
    va_start(ap, count);
    for(size_t i = 0; i < count; i++) {
        ListItem item;
        switch(item_type) {
            case SW_INT:
                item.sw_int = va_arg(ap, int64_t);
                break;
            case SW_BOOL:
                // apparently its better to do int instead of bool
                item.sw_bool = va_arg(ap, int);
                break;
            case SW_UNIT:
                item.sw_unit = va_arg(ap, int);
                break;
            case SW_RC:
                item.sw_rc = *alloc(va_arg(ap, RC *));
                break;
        }
        list->items[i] = item;
    }
    va_end(ap);
}
