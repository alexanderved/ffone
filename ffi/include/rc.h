#ifndef _FFONE_RC_H
#define _FFONE_RC_H

#include <stdlib.h>
#include <stdbool.h>
#include <pthread.h>

#define ffone_rc(type) type *

typedef void (*ffone_rc_dtor_t)(void *);

ffone_rc(void) ffone_rc_alloc(size_t size, ffone_rc_dtor_t dtor);
ffone_rc(void) ffone_rc_alloc0(size_t size, ffone_rc_dtor_t dtor);

#define ffone_rc_new(type) ((ffone_rc(type))ffone_rc_alloc(sizeof(type), NULL))
#define ffone_rc_new0(type) ((ffone_rc(type))ffone_rc_alloc0(sizeof(type), NULL))

#define ffone_rc_new_with_dtor(type, dtor) ((ffone_rc(type))ffone_rc_alloc(sizeof(type), (dtor)))
#define ffone_rc_new_with_dtor0(type, dtor) ((ffone_rc(type))ffone_rc_alloc0(sizeof(type), (dtor)))

void ffone_rc_set_dtor(void *rc, ffone_rc_dtor_t dtor);

ffone_rc(void) ffone_rc_ref(void *rc);
void ffone_rc_unref(ffone_rc(void) rc);

void ffone_rc_lock(void *rc);
void ffone_rc_unlock(void *rc);

int ffone_rc_cond_wait(void *rc, pthread_cond_t *cond);

#endif /* _FFONE_RC_H */