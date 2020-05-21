#include <stdbool.h>
#include <string.h>
#include <stdlib.h>
#include <assert.h>
#include <stdint.h>

#include "rc.h"
#include "strings.h"

// TODO: properly handle UTF-8??

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

void index_string1(RC *dest, RC *src, int64_t idx) {
    index_string2(dest, src, idx, idx+1);
}

// TODO: fancy things ala python string indexing
void index_string2(RC *dest, RC *src, int64_t low, int64_t high) {
    String *src_string = (String *)src->reference;
    assert(low >= 0 && high >= 0);
    assert(low <= high && (size_t)high <= src_string->length);

    String *str = malloc(sizeof(String));
    str->length = high - low;
    str->string = malloc(high - low + 1);
    for(int i = low; i < high; i++)
        str->string[i - low] = src_string->string[i];
    str->string[high - low] = '\0';

    destroy_noref(src);
    new(dest, str, (Destructor) destroy_string);
}

int64_t length_string(RC *s) {
    String *str = (String *)s->reference;
    int64_t length = (int64_t)str->length;
    destroy_noref(s);
    return length;
}
