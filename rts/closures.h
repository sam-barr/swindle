typedef union Env {
    int64_t n;
    bool b;
    bool u;
    RC *rc;
} Env;

typedef enum EnvType {
    E_INT,
    E_BOOL,
    E_UNIT,
    E_RC,
    E_END,
} EnvType;

typedef Env (*ClosureFn)(Env *, ...);

typedef struct Closure {
    Env *env;
    EnvType *env_type;
    ClosureFn fn;
} Closure;
