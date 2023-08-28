#ifndef _FFONE_RC_H
#define _FFONE_RC_H

#include <stdlib.h>
#include <stdbool.h>

typedef void (*ffone_rc_dtor_t)(void *);

void *ffone_rc_alloc(size_t size, ffone_rc_dtor_t dtor);
void *ffone_rc_alloc0(size_t size, ffone_rc_dtor_t dtor);

#define ffone_rc_new(type) ((type *)ffone_rc_alloc(sizeof(type), NULL))
#define ffone_rc_new0(type) ((type *)ffone_rc_alloc0(sizeof(type), NULL))

#define ffone_rc_new_with_dtor(type, dtor) ((type *)ffone_rc_alloc(sizeof(type), (dtor)))
#define ffone_rc_new_with_dtor0(type, dtor) ((type *)ffone_rc_alloc0(sizeof(type), (dtor)))

void ffone_rc_set_dtor(void *rc, ffone_rc_dtor_t dtor);

void *ffone_rc_ref(void *rc);
void ffone_rc_unref(void *rc);

void *ffone_rc_ref_weak(void *rc);
void ffone_rc_unref_weak(void *rc);

bool ffone_rc_is_destructed(void *rc);

#endif /* _FFONE_RC_H */