/*
 * Native runtime for the MITA MLIR backend.
 *
 * Value representation and behavior mirror the bytecode VM (see
 * src/vm/value.rs and src/vm/builtins.rs). Allocation is via malloc and is
 * never freed; the MVP accepts leaks (no GC). Fatal conditions — allocation
 * failure, type errors, division by zero, arity mismatch — call mita_panic
 * and abort, matching the VM's panic behavior.
 */

#include "mita_runtime.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static mita_value make_tagged(int64_t tag, int64_t payload) {
    mita_value v;
    v.tag = tag;
    v.payload = payload;
    return v;
}

_Noreturn void mita_panic(const char *msg) {
    fprintf(stderr, "mita: %s\n", msg);
    abort();
}

static void *xmalloc(size_t n) {
    void *p = malloc(n);
    if (p == NULL) {
        mita_panic("out of memory");
    }
    return p;
}

mita_value mita_nil(void) {
    return make_tagged(MITA_TAG_NIL, 0);
}

mita_value mita_bool(int64_t v) {
    return make_tagged(MITA_TAG_BOOL, v ? 1 : 0);
}

mita_value mita_int(int64_t v) {
    return make_tagged(MITA_TAG_INT, v);
}

static mita_value make_string(int64_t tag, const char *data, int64_t len) {
    if (len < 0) {
        len = (int64_t)strlen(data);
    }
    struct mita_string *s = (struct mita_string *)xmalloc(sizeof(struct mita_string) + (size_t)len + 1);
    s->len = len;
    if (len > 0) {
        memcpy(s->data, data, (size_t)len);
    }
    s->data[len] = '\0';
    return make_tagged(tag, (int64_t)(intptr_t)s);
}

mita_value mita_string(const char *data, int64_t len) {
    return make_string(MITA_TAG_STRING, data, len);
}

mita_value mita_symbol(const char *data, int64_t len) {
    return make_string(MITA_TAG_SYMBOL, data, len);
}

mita_value mita_cons(mita_value car, mita_value cdr) {
    struct mita_cons *c = (struct mita_cons *)xmalloc(sizeof(struct mita_cons));
    c->car = car;
    c->cdr = cdr;
    return make_tagged(MITA_TAG_CONS, (int64_t)(intptr_t)c);
}

mita_value mita_car(mita_value v) {
    if (v.tag != MITA_TAG_CONS) {
        return mita_nil();
    }
    return ((struct mita_cons *)(intptr_t)v.payload)->car;
}

mita_value mita_cdr(mita_value v) {
    if (v.tag != MITA_TAG_CONS) {
        return mita_nil();
    }
    return ((struct mita_cons *)(intptr_t)v.payload)->cdr;
}

static int64_t as_number(mita_value v, const char *op) {
    if (v.tag != MITA_TAG_INT) {
        char msg[128];
        snprintf(msg, sizeof(msg), "%s: expected number, got tag %lld",
                 op, (long long)v.tag);
        mita_panic(msg);
    }
    return v.payload;
}

mita_value mita_add(mita_value a, mita_value b) {
    return mita_int(as_number(a, "add") + as_number(b, "add"));
}

mita_value mita_sub(mita_value a, mita_value b) {
    return mita_int(as_number(a, "sub") - as_number(b, "sub"));
}

mita_value mita_mul(mita_value a, mita_value b) {
    return mita_int(as_number(a, "mul") * as_number(b, "mul"));
}

mita_value mita_div(mita_value a, mita_value b) {
    int64_t x = as_number(a, "div");
    int64_t y = as_number(b, "div");
    if (y == 0) {
        mita_panic("div: division by zero");
    }
    return mita_int(x / y);
}

static int string_equal(mita_value a, mita_value b) {
    struct mita_string *sa = (struct mita_string *)(intptr_t)a.payload;
    struct mita_string *sb = (struct mita_string *)(intptr_t)b.payload;
    return sa->len == sb->len && memcmp(sa->data, sb->data, (size_t)sa->len) == 0;
}

/* Structural equality, matching the VM's shato. */
static int mita_equal(mita_value a, mita_value b) {
    /* nil is equal to the false boolean, matching the VM. */
    if (a.tag == MITA_TAG_NIL && b.tag == MITA_TAG_BOOL && b.payload == 0) {
        return 1;
    }
    if (b.tag == MITA_TAG_NIL && a.tag == MITA_TAG_BOOL && a.payload == 0) {
        return 1;
    }
    if (a.tag != b.tag) {
        return 0;
    }
    switch (a.tag) {
    case MITA_TAG_NIL:
        return 1;
    case MITA_TAG_BOOL:
    case MITA_TAG_INT:
        return a.payload == b.payload;
    case MITA_TAG_STRING:
    case MITA_TAG_SYMBOL:
        return string_equal(a, b);
    case MITA_TAG_CONS: {
        struct mita_cons *ca = (struct mita_cons *)(intptr_t)a.payload;
        struct mita_cons *cb = (struct mita_cons *)(intptr_t)b.payload;
        return mita_equal(ca->car, cb->car) && mita_equal(ca->cdr, cb->cdr);
    }
    default:
        return 0;
    }
}

mita_value mita_eq(mita_value a, mita_value b) {
    return mita_bool(mita_equal(a, b));
}

mita_value mita_ne(mita_value a, mita_value b) {
    return mita_bool(!mita_equal(a, b));
}

mita_value mita_lt(mita_value a, mita_value b) {
    return mita_bool(as_number(a, "lt") < as_number(b, "lt"));
}

mita_value mita_gt(mita_value a, mita_value b) {
    return mita_bool(as_number(a, "gt") > as_number(b, "gt"));
}

mita_value mita_le(mita_value a, mita_value b) {
    return mita_bool(as_number(a, "le") <= as_number(b, "le"));
}

mita_value mita_ge(mita_value a, mita_value b) {
    return mita_bool(as_number(a, "ge") >= as_number(b, "ge"));
}

int64_t mita_truthy(mita_value v) {
    return (v.tag == MITA_TAG_BOOL && v.payload == 1) ? 1 : 0;
}

mita_value mita_make_closure(void *fn, int64_t arity) {
    struct mita_closure *c = (struct mita_closure *)xmalloc(sizeof(struct mita_closure));
    c->fn = (mita_fn)fn;
    c->arity = arity;
    return make_tagged(MITA_TAG_CLOSURE, (int64_t)(intptr_t)c);
}

mita_value mita_call(mita_value fn, mita_value *args, int64_t argc) {
    if (fn.tag != MITA_TAG_CLOSURE) {
        mita_panic("call: expected closure");
    }
    struct mita_closure *c = (struct mita_closure *)(intptr_t)fn.payload;
    if (c->arity != argc) {
        mita_panic("call: arity mismatch");
    }
    return c->fn(args, argc);
}

static void print_string(const struct mita_string *s) {
    fwrite(s->data, 1, (size_t)s->len, stdout);
}

static void print_value(mita_value v);
static void print_cons_body(const struct mita_cons *c);

/* Matches Value's Display in src/vm/value.rs. */
static void print_value(mita_value v) {
    switch (v.tag) {
    case MITA_TAG_NIL:
        fputs("nil", stdout);
        break;
    case MITA_TAG_BOOL:
        fputs(v.payload ? "da" : "nye", stdout);
        break;
    case MITA_TAG_INT:
        printf("%lld", (long long)v.payload);
        break;
    case MITA_TAG_STRING:
        putchar('"');
        print_string((const struct mita_string *)(intptr_t)v.payload);
        putchar('"');
        break;
    case MITA_TAG_SYMBOL:
        print_string((const struct mita_string *)(intptr_t)v.payload);
        break;
    case MITA_TAG_CONS: {
        const struct mita_cons *c = (const struct mita_cons *)(intptr_t)v.payload;
        putchar('(');
        print_cons_body(c);
        putchar(')');
        break;
    }
    case MITA_TAG_CLOSURE:
        fputs("<closure>", stdout);
        break;
    default:
        fputs("<unknown>", stdout);
        break;
    }
}

static void print_cons_body(const struct mita_cons *c) {
    print_value(c->car);
    mita_value cdr = c->cdr;
    if (cdr.tag == MITA_TAG_NIL) {
        return;
    }
    if (cdr.tag == MITA_TAG_CONS) {
        putchar(' ');
        print_cons_body((const struct mita_cons *)(intptr_t)cdr.payload);
    } else {
        fputs(" . ", stdout);
        print_value(cdr);
    }
}

mita_value mita_print_value(mita_value v) {
    print_value(v);
    putchar('\n');
    return mita_nil();
}

mita_value mita_print(mita_value *args, int64_t argc) {
    for (int64_t i = 0; i < argc; i++) {
        if (i > 0) {
            putchar(' ');
        }
        print_value(args[i]);
    }
    putchar('\n');
    return mita_nil();
}
