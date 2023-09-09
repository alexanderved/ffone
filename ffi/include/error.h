#ifndef _FFONE_ERROR_H
#define _FFONE_ERROR_H

#include <stddef.h>
#include <string.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdarg.h>

#define FFONE_COMMA ,
#define FFONE_SINGLE_ARG(...) __VA_ARGS__

#define FFONE_ON_FAILURE(cond, cmd) do { \
    if (!(cond)) {                       \
        cmd;                             \
    }                                    \
} while (0)

#define FFONE_RETURN_VAL_ON_FAILURE(cond, val) FFONE_ON_FAILURE(cond, return val)
#define FFONE_RETURN_ON_FAILURE(cond) FFONE_RETURN_VAL_ON_FAILURE(cond, )

#define FFONE_GOTO_ON_FAILURE(cond, label) FFONE_ON_FAILURE(cond, goto label)

static inline void ffone_assert(bool cond) {
    FFONE_ON_FAILURE(cond, abort());
}

static inline int ffone_error_make_tag(const char *s) {
    size_t str_len = strlen(s);
    size_t size = str_len < sizeof(int) ? str_len : sizeof(int);

    int tag = 0;
    memcpy(&tag, s, size);

    return tag < 0 ? tag : -tag;
}

#define FFONE_ERROR(err) (err)
#define FFONE_SUCCESS 0
#define FFONE_ERROR_CUSTOM (ffone_error_make_tag("cust"))
#define FFONE_ERROR_INVALID_ARG (ffone_error_make_tag("inar"))
#define FFONE_ERROR_BAD_STATE (ffone_error_make_tag("stat"))
#define FFONE_ERROR_BAD_ALLOC (ffone_error_make_tag("allo"))

#endif /* _FFONE_ERROR_H */