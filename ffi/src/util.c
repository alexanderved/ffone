#include "util.h"

#include <stdio.h>
#include <stdarg.h>
#include <stdlib.h>

#ifdef _WIN32
#include <process.h>

#ifdef getpid
#undef getpid
#endif /* getpid */

#define getpid _getpid

#else /* _WIN32 */
#include <unistd.h>
#endif /* _WIN32 */

char *ffone_format_str(const char *fmt, ...) {
    va_list probe_args;
    va_list args;

    va_start(probe_args, fmt);
    va_copy(args, probe_args);

    size_t buf_len = vsnprintf(NULL, 0, fmt, probe_args);
    va_end(probe_args);

    char *buf = calloc(buf_len + 1, sizeof(char));
    if (!buf) {
        return NULL;
    }

    vsprintf(buf, fmt, args);

    return buf;
}

int ffone_get_pid() {
    return (int)getpid();
}