#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
#include "rc.h"
#include "strings.h"

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

void print_line() {
    printf("\n");
}
