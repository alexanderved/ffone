#define _XOPEN_SOURCE 500 

#include "stream.h"

#include "error.h"

#include <stdio.h>
#include <string.h>
#include <pthread.h>
#include <unistd.h>

#define MAX_BYTES_BUFFER 600

static void success_cb(pa_stream *p, int success, void *userdata);

static void ffone_pa_stream_drain_locked(FFonePAStream *stream);
uint64_t ffone_pa_stream_get_time_locked(FFonePAStream *stream);

struct FFonePAStream {
    ffone_rc(FFonePACore) core; /* const, nonnull */
    ffone_rc(RawAudioQueue) queue; /* const, nonnull */

    ffone_rc(FFonePAVirtualSink) sink; /* const, nonnull */
    ffone_rc(FFonePAVirtualSource) source; /* const, nonnull */

    pa_stream *stream;
    StreamFlags flags;

    uint32_t sample_rate;
    RawAudioFormat format;

    uint64_t time_base;

    pthread_t update_thread; /* const */
    pthread_cond_t write_cond; /* const */
};

static void stream_dtor(void *opaque);

static pa_stream *new_pa_stream(
    FFonePACore *core,
    uint32_t sample_rate,
    RawAudioFormat format
);
static int connect_pa_stream(pa_stream *stream, FFonePACore *core, FFonePAStream *s);

static void *stream_update_thread(void *userdata);

ffone_rc(FFonePAStream) ffone_pa_stream_new(
    FFonePACore *core,
    RawAudioQueue *queue
) {
    FFONE_RETURN_VAL_ON_FAILURE(core && queue, NULL);

    ffone_rc(FFonePAStream) stream = ffone_rc_new0(FFonePAStream);
    FFONE_RETURN_VAL_ON_FAILURE(stream, NULL);

    FFONE_GOTO_ON_FAILURE(stream->core = ffone_rc_ref(core), rc_ref_error);
    FFONE_GOTO_ON_FAILURE(stream->queue = ffone_rc_ref(queue), rc_ref_error);

    FFONE_GOTO_ON_FAILURE(stream->sink = ffone_pa_virtual_sink_new(core), rc_ref_error);
    FFONE_GOTO_ON_FAILURE(
        stream->source = ffone_pa_virtual_source_new(core, stream->sink),
        virtual_source_new_error
    );

    FFONE_GOTO_ON_FAILURE(
        pthread_cond_init(&stream->write_cond, NULL) == 0,
        cond_var_init_error
    );

    FFONE_GOTO_ON_FAILURE(
        pthread_create(&stream->update_thread, NULL, stream_update_thread, stream) == 0,
        thread_create_error
    );

    stream->sample_rate = FFONE_DEFAULT_SAMPLE_RATE;
    stream->format = FFONE_DEFAULT_AUDIO_FORMAT;

    stream->time_base = 0;
    
    ffone_rc_lock(stream);
    ffone_pa_core_loop_lock(stream->core);

    FFONE_GOTO_ON_FAILURE(
        stream->stream = new_pa_stream(core, stream->sample_rate, stream->format),
        new_pa_stream_error
    );

    FFONE_GOTO_ON_FAILURE(
        connect_pa_stream(stream->stream, core, stream) == 0,
        connect_pa_stream_error
    );

    ffone_rc_set_dtor(stream, stream_dtor);

    ffone_pa_core_loop_unlock(stream->core);
    ffone_rc_unlock(stream);

    return stream;
connect_pa_stream_error:
    pa_stream_unref(stream->stream);
new_pa_stream_error:
    stream->flags |= FFONE_STREAM_FLAG_DESTRUCTING;

    ffone_pa_core_loop_unlock(stream->core);
    ffone_rc_unlock(stream);

    pthread_cond_signal(&stream->write_cond);
    pthread_join(stream->update_thread, NULL);
thread_create_error:
    pthread_cond_destroy(&stream->write_cond);
cond_var_init_error:
    if (stream->source) ffone_rc_unref(stream->source);
virtual_source_new_error:
    if (stream->sink) ffone_rc_unref(stream->sink);
rc_ref_error:
    if (stream->queue) ffone_rc_unref(stream->queue);
    if (stream->core) ffone_rc_unref(stream->core);

    if (stream) ffone_rc_unref(stream);

    return NULL;
}

static void stream_dtor(void *opaque) {
    FFonePAStream *stream = opaque;
    FFONE_RETURN_ON_FAILURE(stream);

    ffone_rc_lock(stream);
    stream->flags |= FFONE_STREAM_FLAG_DESTRUCTING;
    ffone_rc_unlock(stream);

    pthread_cond_signal(&stream->write_cond);
    pthread_join(stream->update_thread, NULL);

    if (stream->stream) {
        ffone_pa_core_loop_lock(stream->core);

        ffone_pa_stream_drain_locked(stream);
        pa_stream_set_write_callback(stream->stream, NULL, NULL);

        pa_stream_disconnect(stream->stream);
        pa_stream_unref(stream->stream);

        ffone_pa_core_loop_unlock(stream->core);
    }
    stream->stream = NULL;

    pthread_cond_destroy(&stream->write_cond);

    if (stream->source) ffone_rc_unref(stream->source);
    stream->source = NULL;

    if (stream->sink) ffone_rc_unref(stream->sink);
    stream->sink = NULL;

    if (stream->core) ffone_rc_unref(stream->core);
    stream->core = NULL;
}

static pa_stream *new_pa_stream(
    FFonePACore *core,
    uint32_t sample_rate,
    RawAudioFormat format
) {
    static pa_sample_format_t raw_audio_format_cast[] = {
        [RawAudioFormat_U8] = PA_SAMPLE_U8,
        [RawAudioFormat_S16LE] = PA_SAMPLE_S16LE,
        [RawAudioFormat_S16BE] = PA_SAMPLE_S16BE,
        [RawAudioFormat_S24LE] = PA_SAMPLE_S24LE,
        [RawAudioFormat_S24BE] = PA_SAMPLE_S24BE,
        [RawAudioFormat_S32LE] = PA_SAMPLE_S32LE,
        [RawAudioFormat_S32BE] = PA_SAMPLE_S32BE,
        [RawAudioFormat_F32LE] = PA_SAMPLE_FLOAT32LE,
        [RawAudioFormat_F32BE] = PA_SAMPLE_FLOAT32BE,
    };

    pa_context *context = ffone_pa_core_get_context(core);
    FFONE_RETURN_VAL_ON_FAILURE(context, NULL);

    const pa_sample_spec sample_spec = {
        .format = raw_audio_format_cast[format],
        .rate = sample_rate,
        .channels = 1,
    };
    pa_channel_map map;
    pa_channel_map_init_mono(&map);

    pa_stream *stream = pa_stream_new(
        context,
        "Audio Input Stream",
        &sample_spec,
        &map
    );

    FFONE_RETURN_VAL_ON_FAILURE(stream, NULL);

    return stream;
}

static void underflow_cb(pa_stream *s, void *userdata) {
    puts("underflow");
    (void) s;
    (void) userdata;
}

static void request_cb(pa_stream *p, size_t nbytes, void *userdata) {
    FFonePAStream *s = userdata;
    FFONE_RETURN_ON_FAILURE(s);

    pthread_cond_signal(&s->write_cond);

    (void)p;
    (void)nbytes;
}

static void stream_state_cb(pa_stream *stream, void *userdata) {
    pa_threaded_mainloop *loop = userdata;

    switch (pa_stream_get_state(stream)) {
        case PA_STREAM_READY:
        case PA_STREAM_FAILED:
        case PA_STREAM_TERMINATED:
            pa_threaded_mainloop_signal(loop, 0);
        default:
            break;
    }
}

static int connect_pa_stream(
    pa_stream *stream,
    FFonePACore *core,
    FFonePAStream *s
) {
    FFONE_RETURN_VAL_ON_FAILURE(stream, FFONE_ERROR_INVALID_ARG);

    int ret;

    const pa_buffer_attr buf_attr = {
        .maxlength = -1,
        .tlength = MAX_BYTES_BUFFER,
        .prebuf = 0,
        .minreq = MAX_BYTES_BUFFER / 3,
        .fragsize = -1,
    };
    pa_stream_flags_t flags = PA_STREAM_INTERPOLATE_TIMING | 
        PA_STREAM_NOT_MONOTONIC | PA_STREAM_AUTO_TIMING_UPDATE |
        PA_STREAM_ADJUST_LATENCY | PA_STREAM_VARIABLE_RATE |
        PA_STREAM_START_CORKED;

    pa_stream_set_underflow_callback(stream, underflow_cb, NULL);
    pa_stream_set_write_callback(stream, request_cb, s);
    pa_stream_set_state_callback(stream, stream_state_cb, ffone_pa_core_get_loop(core));

    FFONE_RETURN_VAL_ON_FAILURE(
        (ret = pa_stream_connect_playback(stream, 
            /* sink->base.name */ NULL, &buf_attr, flags, NULL, NULL)) == 0,
        FFONE_ERROR(ret)
    );

    pa_stream_state_t state = PA_STREAM_UNCONNECTED;
    while ((state = pa_stream_get_state(stream)) != PA_STREAM_READY) {
        if (state == PA_STREAM_FAILED || state == PA_STREAM_TERMINATED) {
            return FFONE_ERROR_CUSTOM;
        }

        ffone_pa_core_loop_wait(core);
    }

    pa_stream_set_state_callback(stream, NULL, NULL);

    return FFONE_SUCCESS;
}

static void try_write_locked(FFonePAStream *stream);
static void fix_outdated_props_locked(FFonePAStream *stream);

static void *stream_update_thread(void *userdata) {
    FFonePAStream *stream = userdata;
    FFONE_RETURN_VAL_ON_FAILURE(stream, NULL);

    ffone_rc_lock(stream);

    while (!(stream->flags & FFONE_STREAM_FLAG_DESTRUCTING)) {
        ffone_rc_cond_wait(stream, &stream->write_cond);

        ffone_pa_core_loop_lock(stream->core);

        try_write_locked(stream);
        fix_outdated_props_locked(stream);

        ffone_pa_core_loop_unlock(stream->core);
    }

    ffone_rc_unlock(stream);

    return NULL;
}

static void try_write_locked(FFonePAStream *stream) {
    FFONE_RETURN_ON_FAILURE(stream);
    FFONE_RETURN_ON_FAILURE(stream->stream);

    RawAudioQueue *queue = stream->queue;
    FFONE_RETURN_ON_FAILURE(queue);

    size_t write_buffer_size = pa_stream_writable_size(stream->stream);
    if (write_buffer_size == 0) {
        return;
    }

    // printf("WRITABLE SIZE: %lu\n", write_buffer_size);

    uint8_t *write_buffer = NULL;
    FFONE_RETURN_ON_FAILURE(pa_stream_begin_write(stream->stream,
        (void **)&write_buffer, &write_buffer_size) == 0 && write_buffer);
    uint8_t *write_buffer_cursor = write_buffer;
    uint8_t *write_buffer_end = write_buffer + write_buffer_size;

    // printf("WRITABLE BUFFER SIZE: %lu\n", write_buffer_size);

    ffone_rc_lock(queue);
    while (write_buffer_cursor < write_buffer_end &&
        ffone_raw_audio_queue_has_bytes_locked(queue))
    {
        size_t read_size = write_buffer_end - write_buffer_cursor;
        bool have_same_props = false;
        ffone_raw_audio_queue_read_bytes_with_props_locked(
            queue,
            write_buffer_cursor,
            &read_size,
            stream->format,
            stream->sample_rate,
            &have_same_props
        );

        if (read_size == 0) {
            if (!have_same_props) {
                stream->flags |= FFONE_STREAM_FLAG_OUTDATED_PROPS;
            }

            break;
        }

        write_buffer_cursor += read_size;
    }
    ffone_rc_unlock(queue);

    if (write_buffer_end - write_buffer_cursor > 0) {
        //puts("SILENCE");
        memset(write_buffer_cursor, 0, write_buffer_end - write_buffer_cursor);
    }

    pa_stream_write(
        stream->stream,
        write_buffer,
        write_buffer_size,
        NULL,
        0,
        PA_SEEK_RELATIVE
    );
}

static void update_sample_rate_locked(
    FFonePAStream *stream,
    uint32_t sample_rate
) {
    FFONE_RETURN_ON_FAILURE(stream);

    if (stream->sample_rate == sample_rate) {
        return;
    }

    ffone_pa_stream_drain_locked(stream);

    int success = -1;
    pa_operation *o = pa_stream_update_sample_rate(
        stream->stream,
        sample_rate,
        success_cb,
        &success
    );
    FFONE_RETURN_ON_FAILURE(o);

    if (ffone_pa_core_execute_operation(stream->core, o) == FFONE_SUCCESS) {
        printf("Stream Setting Sample Rate: %d\n", success);

        if (success) {
            stream->sample_rate = sample_rate;
        }
    }
}

static void ffone_pa_stream_play_locked(FFonePAStream *stream);
static void update_pa_stream_locked(
    FFonePAStream *stream,
    uint32_t sample_rate,
    RawAudioFormat format
) {
    FFONE_RETURN_ON_FAILURE(stream);

    if (stream->stream) {
        stream->time_base = ffone_pa_stream_get_time_locked(stream);

        ffone_pa_stream_drain_locked(stream);

        pa_stream_disconnect(stream->stream);
        pa_stream_unref(stream->stream);

        stream->stream = NULL;
    }

    FFONE_RETURN_ON_FAILURE(stream->stream = new_pa_stream(
        stream->core, sample_rate, format));
    FFONE_GOTO_ON_FAILURE(
        connect_pa_stream(stream->stream, stream->core, stream) == 0,
        connect_pa_stream_error
    );

    if (stream->flags & FFONE_STREAM_FLAG_PLAYING) {
        ffone_pa_stream_play_locked(stream);
    }

    stream->sample_rate = sample_rate;
    stream->format = format;

    return;
connect_pa_stream_error:
    pa_stream_unref(stream->stream);
    stream->stream = NULL;
}

static void update_props_locked(
    FFonePAStream *stream,
    uint32_t sample_rate,
    RawAudioFormat format
) {
    FFONE_RETURN_ON_FAILURE(stream);

    if (stream->format != format) {
        update_pa_stream_locked(stream, sample_rate, format);
    } else if (stream->sample_rate != sample_rate) {
        update_sample_rate_locked(stream, sample_rate);
    }
}

static void fix_outdated_props_locked(FFonePAStream *stream) {
    FFONE_RETURN_ON_FAILURE(stream);

    if (!stream->stream || stream->flags & FFONE_STREAM_FLAG_OUTDATED_PROPS) {
        RawAudioQueue *queue = stream->queue;
        bool can_update = true;

        RawAudioFormat new_format;
        can_update &= ffone_raw_audio_queue_front_buffer_format(queue, &new_format);

        uint32_t new_sample_rate;
        can_update &= ffone_raw_audio_queue_front_buffer_sample_rate(queue, &new_sample_rate);

        if (can_update) {
            update_props_locked(stream, new_sample_rate, new_format);
            stream->flags &= ~FFONE_STREAM_FLAG_OUTDATED_PROPS;
        }
    }
}

struct SuccessCallbackResult {
    pa_threaded_mainloop *loop;
    int success;
};

static void success_cb(pa_stream *p, int success, void *userdata) {
    struct SuccessCallbackResult *res = userdata;

    if (res) {
        res->success = success;
        pa_threaded_mainloop_signal(res->loop, 0);
    }

    (void)p;
}

static void ffone_pa_stream_play_locked(FFonePAStream *stream) {
    FFONE_RETURN_ON_FAILURE(stream);

    struct SuccessCallbackResult res = {
        .loop = ffone_pa_core_get_loop(stream->core),
        .success = -1,
    };

    if (stream->stream && pa_stream_is_corked(stream->stream) > 0) {
        pa_operation *o = pa_stream_cork(stream->stream, 0, success_cb, &res);

        if (o && ffone_pa_core_execute_operation(stream->core, o) == FFONE_SUCCESS) {
            printf("Stream Started Playing: %d\n", res.success);
            
            if (res.success > 0) {
                stream->flags |= FFONE_STREAM_FLAG_PLAYING;
            }
        }
    }
}

void ffone_pa_stream_play(FFonePAStream *stream) {
    FFONE_RETURN_ON_FAILURE(stream);

    ffone_rc_lock(stream);
    ffone_pa_core_loop_lock(stream->core);

    ffone_pa_stream_play_locked(stream);

    ffone_pa_core_loop_unlock(stream->core);
    ffone_rc_unlock(stream);
}

static void ffone_pa_stream_drain_locked(FFonePAStream *stream) {
    FFONE_RETURN_ON_FAILURE(stream);

    if (pa_stream_is_corked(stream->stream) > 0) {
        return;
    }

    pa_usec_t latency = 0;
    int negative = 0;
    while (pa_stream_get_latency(stream->stream, &latency, &negative) != 0) {
        struct SuccessCallbackResult res = {
            .loop = ffone_pa_core_get_loop(stream->core),
            .success = -1,
        };
        
        pa_operation *o = pa_stream_update_timing_info(stream->stream, success_cb, &res);
        FFONE_ON_FAILURE(o, continue);

        ffone_pa_core_execute_operation(stream->core, o);
    }

    if (negative == 0) {
        usleep(latency);
    }

    struct SuccessCallbackResult res = {
        .loop = ffone_pa_core_get_loop(stream->core),
        .success = -1,
    };

    pa_operation *o = pa_stream_drain(stream->stream, success_cb, &res);
    FFONE_RETURN_ON_FAILURE(o);

    if (ffone_pa_core_execute_operation(stream->core, o) == FFONE_SUCCESS) {
        printf("Stream Drained: %d\n", res.success);
    }
}

uint64_t ffone_pa_stream_get_time_locked(FFonePAStream *stream) {
    FFONE_RETURN_VAL_ON_FAILURE(stream, 0);
    FFONE_RETURN_VAL_ON_FAILURE(stream->stream, 0);

    uint64_t time_base = stream->time_base;

    pa_usec_t usec;
    FFONE_RETURN_VAL_ON_FAILURE(
        pa_stream_get_time(stream->stream, &usec) == 0,
        time_base
    );

    return time_base + usec;
}

uint64_t ffone_pa_stream_get_time(FFonePAStream *stream) {
    FFONE_RETURN_VAL_ON_FAILURE(stream, 0);

    ffone_rc_lock(stream);
    ffone_pa_core_loop_lock(stream->core);

    uint64_t time = ffone_pa_stream_get_time_locked(stream);

    ffone_pa_core_loop_unlock(stream->core);
    ffone_rc_unlock(stream);

    return time;
}
