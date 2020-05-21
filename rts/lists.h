typedef enum ItemType {
    SW_INT,
    SW_BOOL,
    SW_UNIT,
    SW_STRING,
    SW_LIST,
} ItemType;

typedef struct List {
    void *items;
    ItemType item_type;
    size_t length;
    size_t capacity;
} List;

void destroy_list(List *list);
void rc_list(RC *rc, ItemType item_type, size_t count, ...);
void index_list(RC *rc, int64_t idx, void *out);
int64_t length_list(RC *l);
