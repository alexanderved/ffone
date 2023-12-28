#include "rc.h"
#include "error.h"

#include <stdalign.h>
#include <stddef.h>
#include <stdatomic.h>
#include <pthread.h>

typedef struct RcHeader {
    alignas(max_align_t) atomic_size_t strong_count;
    _Atomic ffone_rc_dtor_t dtor;

    pthread_mutex_t mutex;
} RcHeader;

static void rc_header_init(RcHeader *rc_header, ffone_rc_dtor_t dtor) {
    if (!rc_header) {
        return;
    }

    atomic_init(&rc_header->strong_count, 1);
    atomic_init(&rc_header->dtor, dtor);

    pthread_mutex_init(&rc_header->mutex, NULL);
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
    atomic_store(&rc_header->dtor, dtor);
}

ffone_rc(void) ffone_rc_ref(ffone_rc_ptr(void) rc) {
    if (!rc) {
        return NULL;
    }

    RcHeader *rc_header = (RcHeader *)rc - 1;
    atomic_fetch_add_explicit(&rc_header->strong_count, 1, memory_order_release);

    return rc;
}

void ffone_rc_unref(ffone_rc(void) rc) {
    if (!rc) {
        return;
    }

    RcHeader *rc_header = (RcHeader *)rc - 1;

    if (atomic_fetch_sub_explicit(&rc_header->strong_count, 1, memory_order_release) == 1) {
        atomic_thread_fence(memory_order_acquire);

        ffone_rc_dtor_t dtor = atomic_load(&rc_header->dtor);
        if (dtor) {
            (dtor)(rc);
        }

        pthread_mutex_destroy(&rc_header->mutex);

        free(rc_header);
    }
}

void ffone_rc_lock(ffone_rc_ptr(void) rc) {
    if (!rc) {
        return;
    }

    RcHeader *rc_header = (RcHeader *)rc - 1;
    /* ffone_assert( */pthread_mutex_lock(&rc_header->mutex)/*  == 0) */;
}

void ffone_rc_unlock(ffone_rc_ptr(void) rc) {
    if (!rc) {
        return;
    }

    RcHeader *rc_header = (RcHeader *)rc - 1;
    /* ffone_assert( */pthread_mutex_unlock(&rc_header->mutex)/*  == 0) */;
}