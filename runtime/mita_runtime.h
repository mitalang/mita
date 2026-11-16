#ifndef MITA_RUNTIME_H
#define MITA_RUNTIME_H

#include <stdint.h>

/*
 * Native runtime for the MITA MLIR backend.
 *
 * Every MITA value is a (tag, payload) pair. The MLIR backend lowers every
 * value to `!llvm.struct<(i64, i64)>`, so this struct must stay exactly two
 * int64_t fields (16 bytes, natural alignment) with no padding surprises on
 * x86-64 / aarch64 / riscv64.
 */

/* Tag values carried in mita_value.tag. */
#define MITA_TAG_NIL 0
#define MITA_TAG_BOOL 1
#define MITA_TAG_INT 2
#define MITA_TAG_STRING 3
#define MITA_TAG_SYMBOL 4
#define MITA_TAG_CONS 5
#define MITA_TAG_CLOSURE 6

typedef struct {
    int64_t tag;
    int64_t payload;
} mita_value;

/* A compiled MITA function: (args, argc) -> result. */
typedef mita_value (*mita_fn)(mita_value *args, int64_t argc);

/* Cons cell: payload points at one of these. */
struct mita_cons {
    mita_value car;
    mita_value cdr;
};

/* String / symbol backing store: payload points at one of these. */
struct mita_string {
    int64_t len;
    char data[];
};

/* Closure: payload points at one of these. */
struct mita_closure {
    mita_fn fn;
    int64_t arity;
};

/* Constructors. */
mita_value mita_nil(void);
mita_value mita_bool(int64_t v);
mita_value mita_int(int64_t v);
mita_value mita_string(const char *data, int64_t len);
mita_value mita_symbol(const char *data, int64_t len);
mita_value mita_cons(mita_value car, mita_value cdr);

/* Accessors. lawa/kucha on non-cons return nil. */
mita_value mita_car(mita_value v);
mita_value mita_cdr(mita_value v);

/* Arithmetic: numbers only; type errors and division by zero abort. */
mita_value mita_add(mita_value a, mita_value b);
mita_value mita_sub(mita_value a, mita_value b);
mita_value mita_mul(mita_value a, mita_value b);
mita_value mita_div(mita_value a, mita_value b);

/* eq/ne are structural (strings/symbols by content); lt/gt/le/ge numeric. */
mita_value mita_eq(mita_value a, mita_value b);
mita_value mita_ne(mita_value a, mita_value b);
mita_value mita_lt(mita_value a, mita_value b);
mita_value mita_gt(mita_value a, mita_value b);
mita_value mita_le(mita_value a, mita_value b);
mita_value mita_ge(mita_value a, mita_value b);

/* Only bool true (tag=1, payload=1) is truthy. */
int64_t mita_truthy(mita_value v);

/* Closures and calls. */
mita_value mita_make_closure(void *fn, int64_t arity);
mita_value mita_call(mita_value fn, mita_value *args, int64_t argc);

/* Printing. mita_print emits args space-separated + newline; mita_print_value
 * emits one value + newline. Both return nil. */
mita_value mita_print(mita_value *args, int64_t argc);
mita_value mita_print_value(mita_value v);

/* Fatal runtime error: print message to stderr and abort. */
_Noreturn void mita_panic(const char *msg);

#endif /* MITA_RUNTIME_H */
