#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>

#include "rc.h"
#include "strings.h"
#include "lists.h"

void print_int(int64_t i) {
    printf("%ld", i);
}

void print_bool(bool b) {
    printf("%s", b ? "true" : "false");
}

void print_unit(__attribute__((unused)) bool _u) {
    printf("()");
}

void print_string(RC *s) {
    String *str = (String *)s->reference;
    printf("%s", str->string);
    destroy_noref(s);
}

void print_list(RC *l) {
    alloc(l); // index_list will destroy a list if no reference is held, so we hold one...
    List *list = (List *)l->reference;

    printf("[");
    for(int64_t i = 0; i < (int64_t)list->length; i++) {
        ListItem item = index_list(l, i);
        switch(list->item_type) {
            case SW_INT:
                print_int(item.n);
                break;
            case SW_BOOL:
                print_bool(item.b);
                break;
            case SW_UNIT:
                print_unit(item.u);
                break;
            case SW_STRING:
                print_string(as_rc(item));
                break;
            case SW_LIST:
                print_list(as_rc(item));
                break;
        }
        if((size_t)i != list->length - 1)
            printf(", ");
    }
    printf("]");

    drop(l); // and then drop it
}

void print_line() {
    printf("\n");
}
