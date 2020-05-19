typedef enum ItemType {
    SW_INT,
    SW_BOOL,
    SW_UNIT,
    SW_RC,
} ItemType;

typedef union ListItem {
    int64_t sw_int;
    bool sw_bool;
    bool sw_unit;
    RC sw_rc;
} ListItem;

typedef struct List {
    ListItem *items;
    ItemType item_type;
    size_t length;
    size_t capacity;
} List;

void destroy_list(List *list);
void new_list(List *list, ItemType item_type, size_t count, ...);
