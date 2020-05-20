#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
#include "rc.h"
#include "strings.h"

void print_int(int64_t i, bool newline) {
    printf("%ld%s", i, newline ? "\n" : "");
}

void print_bool(bool b, bool newline) {
    printf("%s%s", b ? "true" : "false", newline ? "\n" : "");
}

void print_unit(__attribute__((unused)) bool _u, bool newline) {
    printf("()%s", newline ? "\n" : "");
}

void print_string(RC *s, bool newline) {
    String *str = (String *)s->reference;
    printf("%s%s", str->string, newline ? "\n" : "");
    destroy_noref(s);
}
