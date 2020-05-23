typedef enum ItemType {
    SW_INT,
    SW_BOOL,
    SW_UNIT,
    SW_STRING,
    SW_LIST,
} ItemType;

typedef union ListItem {
    int64_t n;
    bool b;
    bool u;
    RC *rc;
} ListItem;

typedef struct List {
    void *items;
    ItemType item_type;
    size_t length;
    size_t capacity;
} List;

void destroy_list(List *list);
void rc_list(RC *rc, ItemType item_type, size_t count, ...);
ListItem index_list(RC *rc, int64_t idx);
int64_t length_list(RC *l);

int64_t as_int(ListItem item);
bool as_bool(ListItem item);
bool as_unit(ListItem item);
RC *as_rc(ListItem item);

void push_(RC *l, ...);
void set_(RC *l, int64_t idx, ...);
void set_varargs_(RC *l, int64_t idx, va_list ap);
