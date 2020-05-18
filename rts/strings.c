#include <stdbool.h>
#include <string.h>

#include "rc.h"

bool streq(RC *s1, RC *s2) {
    char *str1 = (char *)s1->reference;
    char *str2 = (char *)s2->reference;
    return strcmp(str1, str2) == 0;
}
