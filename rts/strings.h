typedef struct String {
    char *string;
    size_t length;
} String;

bool streq(RC *s1, RC *s2);
void append(RC *rc, RC *s1, RC *s2);
void destroy_string(String *s);
void rc_string(RC *rc, char *s);
void index_string1(RC *dest, RC *src, int64_t idx);
void index_string2(RC *dest, RC *src, int64_t low, int64_t high);
