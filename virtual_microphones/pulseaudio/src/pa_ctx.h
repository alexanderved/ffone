#ifndef _FFONE_PA_CTX_H
#define _FFONE_PA_CTX_H

#include "stream.h"

#include "queue.h"

typedef struct PAContext PAContext;

ffone_rc(PAContext) pa_ctx_new(RawAudioQueue *queue);

pa_context *pa_ctx_get_context(ffone_rc_ptr(PAContext) pa_ctx);
pa_mainloop *pa_ctx_get_loop(ffone_rc_ptr(PAContext) pa_ctx);
ffone_rc_ptr(RawAudioQueue) pa_ctx_get_queue(ffone_rc_ptr(PAContext) pa_ctx);

int pa_ctx_execute_operation(ffone_rc_ptr(PAContext) pa_ctx, pa_operation *o);

int pa_ctx_load_virtual_device(
    ffone_rc_ptr(PAContext) pa_ctx,
    const char *module,
    const char *args,
    pa_context_index_cb_t cb,
    void *userdata
);
int pa_ctx_unload_virtual_device(
    ffone_rc_ptr(PAContext) pa_ctx,
    uint32_t idx,
    pa_context_success_cb_t cb,
    void *userdata
);

int pa_ctx_iterate(ffone_rc_ptr(PAContext) pa_ctx, int block);
int pa_ctx_update(ffone_rc_ptr(PAContext) pa_ctx, int block);

#endif /* _FFONE_PA_CTX_H */