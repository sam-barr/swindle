typedef void (*Destructor)(void *);

typedef struct RC {
    Destructor destructor;
    void *reference;
    int *count;
} RC;


void drop(RC *rc);
RC *copy(RC *rc);
void new(RC *rc, void *reference, Destructor destructor);
