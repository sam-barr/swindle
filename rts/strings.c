#include <stdbool.h>
#include <string.h>
#include <stdlib.h>

#include "rc.h"

bool streq(RC *s1, RC *s2) {
    char *str1 = (char *)s1->reference;
    char *str2 = (char *)s2->reference;
    destroy_noref(s1);
    destroy_noref(s2);
    return strcmp(str1, str2) == 0;
}

void append(RC *rc, RC *s1, RC *s2) {
    char *str1 = (char *)s1->reference;
    char *str2 = (char *)s2->reference;
    char *res = malloc(strlen(str1) + strlen(str2) + 1);
    strcpy(res, str1);
    strcat(res, str2);
    destroy_noref(s1);
    destroy_noref(s2);
    new(rc, res, free);
}
