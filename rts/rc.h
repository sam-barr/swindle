typedef void (*Destructor)(void *);

typedef struct RC {
    Destructor destructor;
    void *reference;
    int *count;
} RC;

void drop(RC **rc);
RC *alloc(RC *rc);
void new(RC *rc, void *reference, Destructor destructor);
void rc_string(RC *rc, char *s);
void uninit(RC *rc);
void destruct_if0(RC *rc);
