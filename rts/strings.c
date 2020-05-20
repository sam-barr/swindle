#include <stdbool.h>
#include <string.h>
#include <stdlib.h>

#include "rc.h"
#include "strings.h"

bool streq(RC *s1, RC *s2) {
    String *string1 = (String *)s1->reference,
           *string2 = (String *)s2->reference;
    bool cmp = strcmp(string1->string, string2->string) == 0;
    destroy_noref(s1);
    destroy_noref(s2);
    return cmp;
}

void append(RC *rc, RC *s1, RC *s2) {
    String *string1 = (String *)s1->reference,
           *string2 = (String *)s2->reference,
           *result  = malloc(sizeof(String));
    result->length = string1->length + string2->length;
    result->string = malloc(string1->length + string2->length + 1);
    strcpy(result->string, string1->string);
    strcat(result->string, string2->string);

    destroy_noref(s1);
    destroy_noref(s2);
    new(rc, result, (Destructor) destroy_string);
}

void destroy_string(String *s) {
    free(s->string);
    free(s);
}

/*
 * Creates a RC for a string (usually a string constant)
 * creates a new, malloced copy of s
 */
void rc_string(RC *rc, char *string) {
    String *s = malloc(sizeof(String));
    s->string = strdup(string);
    s->length = strlen(string);
    new(rc, s, (Destructor) destroy_string);
}
