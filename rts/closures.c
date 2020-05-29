#include <stdlib.h>
#include <stdbool.h>
#include <stdarg.h>
#include <stdio.h>

#include "rc.h"
#include "closures.h"

void destroy_closure(Closure *closure) {
    for(int i = 0; closure->env_type[i] != E_END; i++) {
        if(closure->env_type[i] == E_RC)
            drop(closure->env[i].rc);
    }
    free(closure->env);
    free(closure->env_type);
    free(closure);
}

Env *get_env(RC *c) {
    Closure *closure = (Closure *)c->reference;
    return closure->env;
}

ClosureFn get_fn(RC *c) {
    Closure *closure = (Closure *)c->reference;
    return closure->fn;
}
