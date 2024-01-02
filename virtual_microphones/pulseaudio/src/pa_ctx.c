#define _XOPEN_SOURCE 500

#include "pa_ctx.h"
#include "core.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <time.h>

#include "util.h"
#include "rc.h"
#include "error.h"

struct FFonePAContext {
    ffone_rc(FFonePACore) core; /* const */

    ffone_rc(FFonePAVirtualSink) sink; /* const */
    ffone_rc(FFonePAVirtualSource) src; /* const */

    ffone_rc(FFonePAStream) stream; /* const */
};

static void ffone_pa_ctx_dtor(void *opaque);

ffone_rc(FFonePAContext) ffone_pa_ctx_new(ffone_rc_ptr(RawAudioQueue) queue) {
    srand((unsigned int)time(0) ^ (unsigned int)ffone_get_pid());

    ffone_rc(FFonePAContext) pa_ctx = ffone_rc_new0(FFonePAContext);
    FFONE_RETURN_VAL_ON_FAILURE(pa_ctx, NULL);

    FFONE_GOTO_ON_FAILURE(pa_ctx->core = ffone_pa_core_new(), error);

    FFONE_GOTO_ON_FAILURE(pa_ctx->sink = ffone_pa_virtual_sink_new(pa_ctx->core), error);
    FFONE_GOTO_ON_FAILURE(
        pa_ctx->src = ffone_pa_virtual_source_new(pa_ctx->core, pa_ctx->sink), error);

    FFONE_GOTO_ON_FAILURE(pa_ctx->stream = ffone_pa_stream_new(pa_ctx->core, pa_ctx->sink,
        ffone_rc_ref(queue), FFONE_DEFAULT_SAMPLE_RATE, FFONE_DEFAULT_AUDIO_FORMAT), error);

    ffone_rc_set_dtor(pa_ctx, ffone_pa_ctx_dtor);

    return pa_ctx;
error:
    if (pa_ctx->stream) ffone_rc_unref(pa_ctx->stream);

    if (pa_ctx->src) ffone_rc_unref(pa_ctx->src);
    if (pa_ctx->sink) ffone_rc_unref(pa_ctx->sink);

    if (pa_ctx->core) ffone_rc_unref(pa_ctx->core);
    if (pa_ctx) ffone_rc_unref(pa_ctx);

    return NULL;
}

static void ffone_pa_ctx_dtor(void *opaque) {
    FFonePAContext *pa_ctx = opaque;
    FFONE_RETURN_ON_FAILURE(pa_ctx);

    if (pa_ctx->stream) ffone_rc_unref(pa_ctx->stream);
    pa_ctx->stream = NULL;

    if (pa_ctx->src) ffone_rc_unref(pa_ctx->src);
    pa_ctx->src = NULL;

    if (pa_ctx->sink) ffone_rc_unref(pa_ctx->sink);
    pa_ctx->sink = NULL;

    if (pa_ctx->core) ffone_rc_unref(pa_ctx->core);
    pa_ctx->core = NULL;

    puts("FFonePAContext dtor");
}

ffone_rc_ptr(FFonePAStream) ffone_pa_ctx_get_stream(ffone_rc_ptr(FFonePAContext) pa_ctx) {
    FFONE_RETURN_VAL_ON_FAILURE(pa_ctx, NULL);

    return pa_ctx->stream;
}

int ffone_pa_ctx_update(ffone_rc_ptr(FFonePAContext) pa_ctx) {
    FFONE_RETURN_VAL_ON_FAILURE(pa_ctx, FFONE_ERROR_INVALID_ARG);

    if (pa_ctx->stream) {
        ffone_pa_stream_update(pa_ctx->stream);
    }

    return 0;
}