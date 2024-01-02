#ifndef _FFONE_PA_CTX_H
#define _FFONE_PA_CTX_H

#include "stream.h"

#include "queue.h"

typedef struct FFonePAContext FFonePAContext;

ffone_rc(FFonePAContext) ffone_pa_ctx_new(RawAudioQueue *queue);
ffone_rc_ptr(FFonePAStream) ffone_pa_ctx_get_stream(ffone_rc_ptr(FFonePAContext) pa_ctx);
int ffone_pa_ctx_update(ffone_rc_ptr(FFonePAContext) pa_ctx);

#endif /* _FFONE_PA_CTX_H */