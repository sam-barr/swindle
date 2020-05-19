typedef void (*Destructor)(void *);

typedef struct RC {
    Destructor destructor;
    void *reference;
    int *count;
} RC;

void drop(RC *rc);
void drop2(RC **rc);
RC *alloc(RC *rc);
void new(RC *rc, void *reference, Destructor destructor);
void rc_string(RC *rc, char *s);
void uninit(RC *rc);
void destroy_noref(RC *rc);
