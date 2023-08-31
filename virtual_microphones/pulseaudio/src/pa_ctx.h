#ifndef _FFONE_PA_CTX_H
#define _FFONE_PA_CTX_H

#include "virtual_device.h"
#include "stream.h"

#include "queue.h"

typedef struct PAContext PAContext;

PAContext *pa_ctx_new(RawAudioQueue *queue);

pa_context *pa_ctx_get_context(PAContext *pa_ctx);
pa_mainloop *pa_ctx_get_loop(PAContext *pa_ctx);
RawAudioQueue *pa_ctx_get_queue(PAContext *pa_ctx);

int pa_ctx_execute_operation(PAContext *pa_ctx, pa_operation *o);

int pa_ctx_load_virtual_device(
    PAContext *pa_ctx,
    const char *module,
    const char *args,
    pa_context_index_cb_t cb,
    void *userdata
);
int pa_ctx_unload_virtual_device(
    PAContext *pa_ctx,
    uint32_t idx,
    pa_context_success_cb_t cb,
    void *userdata
);

int pa_ctx_iterate(PAContext *pa_ctx, int block);
int pa_ctx_update(PAContext *pa_ctx, int block);

#endif /* _FFONE_PA_CTX_H */