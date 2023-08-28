#ifndef _FFONE_UTIL_H
#define _FFONE_UTIL_H

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

char *ffone_format_str(const char *fmt, ...);
int ffone_get_pid();

#endif /* _FFONE_UTIL_H */