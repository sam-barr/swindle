#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
#include "rc.h"

void print_int(int64_t i, bool newline) {
    printf("%ld%s", i, newline ? "\n" : "");
}

void print_string(char *s, bool newline) {
    printf("%s%s", s, newline ? "\n" : "");
}

void print_bool(bool b, bool newline) {
    printf("%s%s", b ? "true" : "false", newline ? "\n" : "");
}

void print_unit(bool _u, bool newline) {
    printf("()%s", newline ? "\n" : "");
}

void _print_string(RC *s, bool newline) {
    printf("%s%s", (char *)s->reference, newline ? "\n" : "");
}
