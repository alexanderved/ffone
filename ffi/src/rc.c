#include "rc.h"

#include <stdalign.h>
#include <stddef.h>

typedef struct RcHeader {
    alignas(max_align_t) size_t strong_count;
    size_t weak_count;
    
    ffone_rc_dtor_t dtor;
} RcHeader;

static void rc_header_init(RcHeader *rc_header, ffone_rc_dtor_t dtor) {
    if (!rc_header) {
        return;
    }

    rc_header->strong_count = 1;
    rc_header->weak_count = 1;

    rc_header->dtor = dtor;
}

ffone_rc(void) ffone_rc_alloc(size_t size, ffone_rc_dtor_t dtor) {
    RcHeader *rc_header = malloc(sizeof(RcHeader) + size);
    if (!rc_header) {
        return NULL;
    }
    rc_header_init(rc_header, dtor);

    return (ffone_rc(void))(rc_header + 1);
}

ffone_rc(void) ffone_rc_alloc0(size_t size, ffone_rc_dtor_t dtor) {
    RcHeader *rc_header = calloc(1, sizeof(RcHeader) + size);
    if (!rc_header) {
        return NULL;
    }
    rc_header_init(rc_header, dtor);

    return (ffone_rc(void))(rc_header + 1);
}

void ffone_rc_set_dtor(ffone_rc_ptr(void) rc, ffone_rc_dtor_t dtor) {
    if (!rc) {
        return;
    }

    RcHeader *rc_header = (RcHeader *)rc - 1;
    rc_header->dtor = dtor;
}

ffone_rc(void) ffone_rc_ref(ffone_rc_ptr(void) rc) {
    if (!rc) {
        return NULL;
    }

    RcHeader *rc_header = (RcHeader *)rc - 1;
    if (rc_header->strong_count == 0) {
        return NULL;
    }
    ++rc_header->strong_count;

    return rc;
}

void ffone_rc_unref(ffone_rc(void) rc) {
    if (!rc) {
        return;
    }

    RcHeader *rc_header = (RcHeader *)rc - 1;
    if (rc_header->strong_count == 0) {
        return;
    }

    if (--rc_header->strong_count == 0) {
        if (rc_header->dtor) {
            (rc_header->dtor)(rc);
        }
        
        if (--rc_header->weak_count == 0) {
            free(rc_header);
        }
    }
}

ffone_weak(void) ffone_rc_ref_weak(ffone_rc_ptr(void) rc) {
    if (!rc) {
        return NULL;
    }

    RcHeader *rc_header = (RcHeader *)rc - 1;
    ++rc_header->weak_count;

    return rc;
}

void ffone_rc_unref_weak(ffone_weak(void) rc) {
    if (!rc) {
        return;
    }

    RcHeader *rc_header = (RcHeader *)rc - 1;
    if (rc_header->weak_count == 0) {
        return;
    }

    if (--rc_header->weak_count == 0 && ffone_rc_is_destructed(rc)) {
        free(rc_header);
    }
}

bool ffone_rc_is_destructed(ffone_rc_ptr(void) rc) {
    if (!rc) {
        return true;
    }

    RcHeader *rc_header = (RcHeader *)rc - 1;
    return rc_header->strong_count == 0;
}