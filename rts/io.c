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
    List *list = (List *)l->reference;

    printf("[");
    for(int64_t i = 0; i < (int64_t)list->length; i++) {
        int64_t n;
        bool b;
        RC rc;
        switch(list->item_type) {
            case SW_INT:
                index_list(list, i, &n);
                print_int(n);
                break;
            case SW_BOOL:
                index_list(list, i, &b);
                print_bool(b);
                break;
            case SW_UNIT:
                print_unit(0);
                break;
            case SW_STRING:
                index_list(list, i, &rc);
                print_string(&rc);
                break;
            case SW_LIST:
                index_list(list, i, &rc);
                print_list(&rc);
                break;
        }
        if(i != list->length - 1)
            printf(", ");
    }
    printf("]");

    destroy_noref(l);
}

void print_line() {
    printf("\n");
}
