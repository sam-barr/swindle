typedef struct String {
    char *string;
    size_t length;
} String;

bool streq(RC *s1, RC *s2);
void append(RC *rc, RC *s1, RC *s2);
void destroy_string(String *s);
void rc_string(RC *rc, char *s);
