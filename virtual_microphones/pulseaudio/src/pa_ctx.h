#ifndef _FFONE_PA_CTX_H
#define _FFONE_PA_CTX_H

#include "stream.h"

#include "queue.h"

typedef struct FFonePAContext FFonePAContext;

ffone_rc(FFonePAContext) ffone_pa_ctx_new(RawAudioQueue *queue);

pa_context *ffone_pa_ctx_get_context(ffone_rc_ptr(FFonePAContext) pa_ctx);
pa_mainloop *ffone_pa_ctx_get_loop(ffone_rc_ptr(FFonePAContext) pa_ctx);
ffone_rc_ptr(RawAudioQueue) ffone_pa_ctx_get_queue(ffone_rc_ptr(FFonePAContext) pa_ctx);

int ffone_pa_ctx_execute_operation(ffone_rc_ptr(FFonePAContext) pa_ctx, pa_operation *o);

int ffone_pa_ctx_load_virtual_device(
    ffone_rc_ptr(FFonePAContext) pa_ctx,
    const char *module,
    const char *args,
    pa_context_index_cb_t cb,
    void *userdata
);
int ffone_pa_ctx_unload_virtual_device(
    ffone_rc_ptr(FFonePAContext) pa_ctx,
    uint32_t idx,
    pa_context_success_cb_t cb,
    void *userdata
);

int ffone_pa_ctx_iterate(ffone_rc_ptr(FFonePAContext) pa_ctx, int block);
int ffone_pa_ctx_update(ffone_rc_ptr(FFonePAContext) pa_ctx, int block);

#endif /* _FFONE_PA_CTX_H */