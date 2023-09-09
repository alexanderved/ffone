#define _XOPEN_SOURCE 500

#include "pa_ctx.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <time.h>

#include "util.h"
#include "rc.h"

struct PAContext {
    RawAudioQueue *queue;

    pa_mainloop *loop;
    pa_mainloop_api *api;

    pa_context *context;

    VirtualSink *sink;
    VirtualSource *src;

    Stream *stream;
};

static void pa_ctx_dtor(void *opaque);

ffone_rc(PAContext) pa_ctx_new(ffone_rc_ptr(RawAudioQueue) queue) {
    srand((unsigned int)time(0) ^ (unsigned int)ffone_get_pid());

    ffone_rc(PAContext) pa_ctx = ffone_rc_new0(PAContext);
    FFONE_RETURN_VAL_ON_FAILURE(pa_ctx, NULL);

    pa_ctx->queue = ffone_rc_ref(queue);
    FFONE_GOTO_ON_FAILURE(pa_ctx->queue, error);

    FFONE_GOTO_ON_FAILURE(pa_ctx->loop = pa_mainloop_new(), error);
    FFONE_GOTO_ON_FAILURE(pa_ctx->api = pa_mainloop_get_api(pa_ctx->loop), error);

    FFONE_GOTO_ON_FAILURE(
        pa_ctx->context = pa_context_new(pa_ctx->api, "ffone_pa_virtual_microphone"),
        error
    );
    
    FFONE_GOTO_ON_FAILURE(
        pa_context_connect(pa_ctx->context, NULL, PA_CONTEXT_NOAUTOSPAWN, NULL) >= 0,
        error
    );

    pa_context_state_t state = PA_CONTEXT_UNCONNECTED;
    while (state != PA_CONTEXT_READY) {
        pa_mainloop_iterate(pa_ctx->loop, 1, NULL);

        state = pa_context_get_state(pa_ctx->context);
        if (state == PA_CONTEXT_FAILED || state == PA_CONTEXT_TERMINATED) goto error;
    }

    FFONE_GOTO_ON_FAILURE(pa_ctx->sink = virtual_sink_new(pa_ctx), error);
    FFONE_GOTO_ON_FAILURE(pa_ctx->src = virtual_source_new(pa_ctx, pa_ctx->sink), error);

    FFONE_GOTO_ON_FAILURE(pa_ctx->stream = stream_new(pa_ctx, pa_ctx->sink,
        FFONE_DEFAULT_SAMPLE_RATE, FFONE_DEFAULT_AUDIO_FORMAT), error);

    ffone_rc_set_dtor(pa_ctx, pa_ctx_dtor);

    return pa_ctx;
error:
    fprintf(stderr, "Failed to create PAContext\n");

    if (pa_ctx->stream) ffone_rc_unref(pa_ctx->stream);

    if (pa_ctx->src) ffone_rc_unref(pa_ctx->src);
    if (pa_ctx->sink) ffone_rc_unref(pa_ctx->sink);

    if (pa_ctx->context) {
        if (pa_context_get_state(pa_ctx->context) == PA_CONTEXT_READY) {
            pa_context_disconnect(pa_ctx->context);
        }

        pa_context_unref(pa_ctx->context);
    }

    if (pa_ctx->loop) pa_mainloop_free(pa_ctx->loop);

    if (pa_ctx->queue) ffone_rc_unref(pa_ctx->queue);
    if (pa_ctx) ffone_rc_unref(pa_ctx);

    fprintf(stderr, "Failed PAContext cleaned\n");

    return NULL;
}

static void pa_ctx_dtor(void *opaque) {
    PAContext *pa_ctx = opaque;
    FFONE_RETURN_ON_FAILURE(pa_ctx);

    if (pa_ctx->stream) ffone_rc_unref(pa_ctx->stream);
    pa_ctx->stream = NULL;

    if (pa_ctx->src) ffone_rc_unref(pa_ctx->src);
    pa_ctx->src = NULL;

    if (pa_ctx->sink) ffone_rc_unref(pa_ctx->sink);
    pa_ctx->sink = NULL;

    if (pa_ctx->context) {
        if (pa_context_get_state(pa_ctx->context) == PA_CONTEXT_READY) {
            pa_context_disconnect(pa_ctx->context);
        }
        pa_context_unref(pa_ctx->context);
        pa_ctx->context = NULL;
    }

    pa_ctx->api = NULL;

    if (pa_ctx->loop) {
        pa_mainloop_quit(pa_ctx->loop, 0);
        pa_mainloop_free(pa_ctx->loop);
        pa_ctx->loop = NULL;
    }

    if (pa_ctx->queue) ffone_rc_unref(pa_ctx->queue);
    pa_ctx->queue = NULL;

    puts("PAContext dtor");
}

pa_context *pa_ctx_get_context(ffone_rc_ptr(PAContext) pa_ctx) {
    FFONE_RETURN_VAL_ON_FAILURE(pa_ctx, NULL);
    FFONE_RETURN_VAL_ON_FAILURE(!ffone_rc_is_destructed(pa_ctx), NULL);

    return pa_ctx->context;
}

pa_mainloop *pa_ctx_get_loop(ffone_rc_ptr(PAContext)pa_ctx) {
    FFONE_RETURN_VAL_ON_FAILURE(pa_ctx, NULL);
    FFONE_RETURN_VAL_ON_FAILURE(!ffone_rc_is_destructed(pa_ctx), NULL);

    return pa_ctx->loop;
}

ffone_rc_ptr(RawAudioQueue) pa_ctx_get_queue(ffone_rc_ptr(PAContext)pa_ctx) {
    FFONE_RETURN_VAL_ON_FAILURE(pa_ctx, NULL);
    FFONE_RETURN_VAL_ON_FAILURE(!ffone_rc_is_destructed(pa_ctx), NULL);

    return pa_ctx->queue;
}

int pa_ctx_execute_operation(ffone_rc_ptr(PAContext) pa_ctx, pa_operation *o) {
    FFONE_RETURN_VAL_ON_FAILURE(pa_ctx && o, -1);
    FFONE_RETURN_VAL_ON_FAILURE(!ffone_rc_is_destructed(pa_ctx), -1);

    pa_operation_state_t state;
    while ((state = pa_operation_get_state(o)) == PA_OPERATION_RUNNING) {
        if (pa_ctx_iterate(pa_ctx, 1) < 0) {
            pa_operation_cancel(o);
            continue;
        }
    }

    pa_operation_unref(o);

    return (state == PA_OPERATION_DONE) ? 0 : -1;
}

int pa_ctx_load_virtual_device(
    ffone_rc_ptr(PAContext) pa_ctx,
    const char *module,
    const char *args,
    pa_context_index_cb_t cb,
    void *userdata
) {
    FFONE_RETURN_VAL_ON_FAILURE(pa_ctx && module && args && cb, -1);
    FFONE_RETURN_VAL_ON_FAILURE(!ffone_rc_is_destructed(pa_ctx) && pa_ctx->context, -1);

    pa_operation *o = pa_context_load_module(
        pa_ctx->context,
        module,
        args,
        cb,
        userdata
    );
    FFONE_RETURN_VAL_ON_FAILURE(o, -1);

    return pa_ctx_execute_operation(pa_ctx, o);
}

int pa_ctx_unload_virtual_device(
    ffone_rc_ptr(PAContext) pa_ctx,
    uint32_t idx,
    pa_context_success_cb_t cb,
    void *userdata
) {
    FFONE_RETURN_VAL_ON_FAILURE(pa_ctx && cb, -1);
    FFONE_RETURN_VAL_ON_FAILURE(!ffone_rc_is_destructed(pa_ctx) && pa_ctx->context, -1);
    FFONE_RETURN_VAL_ON_FAILURE(idx != VIRTUAL_DEVICE_INDEX_NONE, 0);

    pa_operation *o = pa_context_unload_module(
        pa_ctx->context,
        idx,
        cb,
        userdata
    );
    FFONE_RETURN_VAL_ON_FAILURE(o, -1);

    return pa_ctx_execute_operation(pa_ctx, o);
}

int pa_ctx_iterate(ffone_rc_ptr(PAContext) pa_ctx, int block) {
    FFONE_RETURN_VAL_ON_FAILURE(pa_ctx, -1);
    FFONE_RETURN_VAL_ON_FAILURE(!ffone_rc_is_destructed(pa_ctx) && pa_ctx->loop, -1);

    return pa_mainloop_iterate(pa_ctx->loop, block, NULL);
}

int pa_ctx_update(ffone_rc_ptr(PAContext) pa_ctx, int block) {
    FFONE_RETURN_VAL_ON_FAILURE(pa_ctx, -1);
    FFONE_RETURN_VAL_ON_FAILURE(pa_ctx->loop, -1);

    int ret = pa_ctx_iterate(pa_ctx, block);

    if (pa_ctx->stream) {
        stream_update(pa_ctx->stream);
    }

    return ret;
}


#include <unistd.h>

void cmain(RawAudioQueue *queue) {
    ffone_rc(PAContext) pa_ctx = pa_ctx_new(queue);
    FFONE_RETURN_ON_FAILURE(pa_ctx);

    /* uint8_t *data = calloc(16, sizeof(uint8_t));
    size_t len = 16;
    printf("%d\n", ffone_raw_audio_queue_read_bytes_formatted(queue, data, &len, U8));

    for (size_t i = 0; i < len; ++i) {
        printf("%hhu ", data[i]);
    }
    putc('\n', stdout); */

    if (pa_ctx->stream) {
        for (int i = 0; i < 1000000000; ++i) {
        //while (1) {
            pa_ctx_update(pa_ctx, 0);

            pa_usec_t usec = 0;
            if (pa_stream_get_time(stream_get_pa_stream(pa_ctx->stream), &usec) == 0) {
                printf("\tAudio Stream Time: %" PRIu64 "ms\n", usec / 1000);
            }
            
            const pa_timing_info *ti = pa_stream_get_timing_info(
                stream_get_pa_stream(pa_ctx->stream));
            
            if (ti) {
                printf("\twrite index: %" PRIi64 "\n", ti->write_index);
                printf("\tread index: %" PRIi64 "\n\n", ti->read_index);
            }



            if (!ffone_raw_audio_queue_has_bytes(queue)) {
                break;
            }

            usleep(100000);
        }
    }

    /* if (pa_ctx->stream) {
        for (int i = 0; i < 1000000; ++i) {
            pa_ctx_iterate(pa_ctx, 0);

            pa_usec_t usec = 0;
            if (pa_stream_get_time(stream_get_pa_stream(pa_ctx->stream), &usec) == 0) {
                printf("\tAudio Stream Time: %" PRIu64 "ms\n", usec / 1000);
            }

            const pa_timing_info *ti = pa_stream_get_timing_info(
                stream_get_pa_stream(pa_ctx->stream));
            
            if (ti) {
                printf("\twrite index: %" PRIi64 "\n", ti->write_index);
                printf("\tread index: %" PRIi64 "\n\n", ti->read_index);

                if (ti->write_index > 0 && ti->read_index > 0
                    && ti->write_index == ti->read_index)
                {
                    break;
                }
            }
        }
    } */

    ffone_rc_unref(queue);
    ffone_rc_unref(pa_ctx);
}