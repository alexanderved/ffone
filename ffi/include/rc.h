#ifndef _FFONE_RC_H
#define _FFONE_RC_H

#include <stdlib.h>
#include <stdbool.h>

#define ffone_rc(type) type *
#define ffone_weak(type) type *
#define ffone_rc_ptr(type) type *

typedef void (*ffone_rc_dtor_t)(void *);

ffone_rc(void) ffone_rc_alloc(size_t size, ffone_rc_dtor_t dtor);
ffone_rc(void) ffone_rc_alloc0(size_t size, ffone_rc_dtor_t dtor);

#define ffone_rc_new(type) ((ffone_rc(type))ffone_rc_alloc(sizeof(type), NULL))
#define ffone_rc_new0(type) ((ffone_rc(type))ffone_rc_alloc0(sizeof(type), NULL))

#define ffone_rc_new_with_dtor(type, dtor) ((ffone_rc(type))ffone_rc_alloc(sizeof(type), (dtor)))
#define ffone_rc_new_with_dtor0(type, dtor) ((ffone_rc(type))ffone_rc_alloc0(sizeof(type), (dtor)))

void ffone_rc_set_dtor(ffone_rc_ptr(void) rc, ffone_rc_dtor_t dtor);

ffone_rc(void) ffone_rc_ref(ffone_rc_ptr(void) rc);
void ffone_rc_unref(ffone_rc(void) rc);

void ffone_rc_lock(ffone_rc_ptr(void) rc);
void ffone_rc_unlock(ffone_rc_ptr(void) rc);

#endif /* _FFONE_RC_H */