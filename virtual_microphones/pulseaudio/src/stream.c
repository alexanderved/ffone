#include "stream.h"

#include "error.h"

#include <stdio.h>
#include <string.h>

#define MAX_BYTES_BUFFER 6000

static void stream_dtor(void *opaque);

static pa_stream *new_pa_stream(
    ffone_rc_ptr(FFonePACore) core,
    uint32_t sample_rate,
    RawAudioFormat format
);
static int connect_pa_stream(pa_stream *stream, ffone_rc_ptr(FFonePACore) core);

static pa_sample_format_t raw_audio_format_to_pa_sample_format_t(RawAudioFormat raw);

static void ffone_pa_stream_success_cb(pa_stream *p, int success, void *userdata);

static void ffone_pa_stream_drain_locked(ffone_rc_ptr(FFonePAStream) stream);
uint64_t ffone_pa_stream_get_time_locked(ffone_rc_ptr(FFonePAStream) stream);

struct FFonePAStream {
    ffone_rc(FFonePACore) core; /* const */
    ffone_rc(FFonePAVirtualSink) sink; /* const */
    ffone_rc(RawAudioQueue) queue; /* const */

    pa_stream *stream;
    StreamFlags flags;

    uint32_t sample_rate;
    RawAudioFormat format;

    uint64_t time_base;
};

ffone_rc(FFonePAStream) ffone_pa_stream_new(
    ffone_rc_ptr(FFonePACore) core,
    ffone_rc_ptr(FFonePAVirtualSink) sink,
    ffone_rc(RawAudioQueue) queue,
    uint32_t sample_rate,
    RawAudioFormat format
) {
    FFONE_RETURN_VAL_ON_FAILURE(core && sink, NULL);

    ffone_rc(FFonePAStream) stream = ffone_rc_new0(FFonePAStream);
    FFONE_RETURN_VAL_ON_FAILURE(stream, NULL);

    FFONE_GOTO_ON_FAILURE(stream->core = ffone_rc_ref(core), rc_ref_error);
    FFONE_GOTO_ON_FAILURE(stream->sink = ffone_rc_ref(sink), rc_ref_error);
    FFONE_GOTO_ON_FAILURE(stream->queue = queue, rc_ref_error);

    ffone_pa_core_loop_lock(stream->core);

    FFONE_GOTO_ON_FAILURE(
        stream->stream = new_pa_stream(core, sample_rate, format),
        new_pa_stream_error
    );

    FFONE_GOTO_ON_FAILURE(
        connect_pa_stream(stream->stream, core) == 0,
        connect_pa_stream_error
    );

    stream->sample_rate = sample_rate;
    stream->format = format;

    stream->time_base = 0;

    ffone_rc_set_dtor(stream, stream_dtor);

    ffone_pa_core_loop_unlock(stream->core);

    return stream;
connect_pa_stream_error:
    pa_stream_unref(stream->stream);
new_pa_stream_error:
    ffone_pa_core_loop_unlock(stream->core);

    if (stream->queue) ffone_rc_unref(stream->queue);
rc_ref_error:
    if (stream->sink) ffone_rc_unref(stream->sink);
    if (stream->core) ffone_rc_unref(stream->core);

    if (stream) ffone_rc_unref(stream);

    return NULL;
}

static void stream_dtor(void *opaque) {
    FFonePAStream *stream = opaque;
    FFONE_RETURN_ON_FAILURE(stream);

    stream->flags = FFONE_STREAM_FLAG_NONE;

    if (stream->stream && stream->core) {
        ffone_pa_core_loop_lock(stream->core);

        ffone_pa_stream_drain_locked(stream);
        pa_stream_set_write_callback(stream->stream, NULL, NULL);

        pa_stream_disconnect(stream->stream);
        pa_stream_unref(stream->stream);

        ffone_pa_core_loop_unlock(stream->core);
    }
    stream->stream = NULL;

    if (stream->sink) ffone_rc_unref(stream->sink);
    stream->sink = NULL;

    if (stream->core) ffone_rc_unref(stream->core);
    stream->core = NULL;
}

static pa_stream *new_pa_stream(
    ffone_rc_ptr(FFonePACore) core,
    uint32_t sample_rate,
    RawAudioFormat format
) {
    pa_context *context = ffone_pa_core_get_context(core);
    FFONE_RETURN_VAL_ON_FAILURE(context, NULL);

    const pa_sample_spec sample_spec = {
        .format = raw_audio_format_to_pa_sample_format_t(format),
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

static void stream_underflow_cb(pa_stream *s, void *userdata)
{
    puts("underflow");
    (void) s;
    (void) userdata;
}

static void stream_state_cb(pa_stream *stream, void *userdata)
{
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

static int connect_pa_stream(pa_stream *stream, ffone_rc_ptr(FFonePACore) core) {
    FFONE_RETURN_VAL_ON_FAILURE(stream, FFONE_ERROR_INVALID_ARG);

    int ret;

    const pa_buffer_attr buf_attr = {
        .maxlength = -1,
        .tlength = MAX_BYTES_BUFFER,
        .prebuf = 0,
        .minreq = -1,
        .fragsize = -1,
    };
    pa_stream_flags_t flags = PA_STREAM_INTERPOLATE_TIMING | 
        PA_STREAM_NOT_MONOTONIC | PA_STREAM_AUTO_TIMING_UPDATE |
        PA_STREAM_ADJUST_LATENCY | PA_STREAM_VARIABLE_RATE;

    pa_stream_set_underflow_callback(stream, stream_underflow_cb, NULL);
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

static void ffone_pa_stream_update_pa_stream_locked(ffone_rc_ptr(FFonePAStream) stream) {
    FFONE_RETURN_ON_FAILURE(stream);

    if (stream->stream) {
        ffone_pa_stream_drain_locked(stream);

        pa_stream_disconnect(stream->stream);
        pa_stream_unref(stream->stream);

        stream->stream = NULL;
    }

    FFONE_RETURN_ON_FAILURE(stream->stream = new_pa_stream(
        stream->core, stream->sample_rate, stream->format));
    FFONE_GOTO_ON_FAILURE(
        connect_pa_stream(stream->stream, stream->core) == 0,
        connect_pa_stream_error
    );

    return;
connect_pa_stream_error:
    pa_stream_unref(stream->stream);
    stream->stream = NULL;
}

static pa_sample_format_t raw_audio_format_to_pa_sample_format_t(RawAudioFormat raw) {
    switch (raw)
    {
    case RawAudioFormat_U8:
        return PA_SAMPLE_U8;
    case RawAudioFormat_S16LE:
        return PA_SAMPLE_S16LE;
    case RawAudioFormat_S16BE:
        return PA_SAMPLE_S16BE;
    case RawAudioFormat_S24LE:
        return PA_SAMPLE_S24LE;
    case RawAudioFormat_S24BE:
        return PA_SAMPLE_S24BE;
    case RawAudioFormat_S32LE:
        return PA_SAMPLE_S32LE;
    case RawAudioFormat_S32BE:
        return PA_SAMPLE_S32BE;
    case RawAudioFormat_F32LE:
        return PA_SAMPLE_FLOAT32LE;
    case RawAudioFormat_F32BE:
        return PA_SAMPLE_FLOAT32BE;
    default:
        return PA_SAMPLE_U8;
    }
}

static void ffone_pa_stream_update_sample_rate_locked(
    ffone_rc_ptr(FFonePAStream) stream,
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
        ffone_pa_stream_success_cb,
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

static void ffone_pa_stream_update_raw_audio_format_locked(
    ffone_rc_ptr(FFonePAStream) stream,
    RawAudioFormat format
) {
    FFONE_RETURN_ON_FAILURE(stream);

    if (stream->format == format) {
        return;
    }
    stream->format = format;

    ffone_pa_stream_update_pa_stream_locked(stream);
}

static void ffone_pa_stream_update_props_locked(
    ffone_rc_ptr(FFonePAStream) stream,
    uint32_t sample_rate,
    RawAudioFormat format
) {
    FFONE_RETURN_ON_FAILURE(stream);

    if (stream->sample_rate == sample_rate && stream->format == format) {
        return;
    }

    if (stream->sample_rate != sample_rate && stream->format == format) {
        ffone_pa_stream_update_sample_rate_locked(stream, sample_rate);

        return;
    }

    stream->time_base = ffone_pa_stream_get_time_locked(stream);
    stream->sample_rate = sample_rate;
    ffone_pa_stream_update_raw_audio_format_locked(stream, format);
}

static void ffone_pa_stream_try_write_locked(ffone_rc_ptr(FFonePAStream) stream) {
    FFONE_RETURN_ON_FAILURE(stream);

    ffone_rc_ptr(RawAudioQueue) queue = stream->queue;
    FFONE_RETURN_ON_FAILURE(queue);

    size_t write_buffer_size = pa_stream_writable_size(stream->stream);
    if (write_buffer_size == 0) {
        return;
    }

    // printf("WRITTABLE SIZE: %lu\n", write_buffer_size);

    uint8_t *write_buffer = NULL;
    FFONE_RETURN_ON_FAILURE(pa_stream_begin_write(stream->stream,
        (void **)&write_buffer, &write_buffer_size) == 0 && write_buffer);
    uint8_t *write_buffer_cursor = write_buffer;
    uint8_t *write_buffer_end = write_buffer + write_buffer_size;

    // printf("WRITTABLE BUFFER SIZE: %lu\n", write_buffer_size);

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

    size_t bytes_written = write_buffer_cursor - write_buffer;
    if (bytes_written == 0) {
        pa_stream_cancel_write(stream->stream);

        return;
    }

    // printf("BYTES WRITTEN: %lu\n\n", bytes_written);

    pa_stream_write(
        stream->stream,
        write_buffer,
        bytes_written,
        NULL,
        0,
        PA_SEEK_RELATIVE
    );
}

void ffone_pa_stream_update(ffone_rc_ptr(FFonePAStream) stream) {
    FFONE_RETURN_ON_FAILURE(stream);

    ffone_rc_lock(stream);
    ffone_pa_core_loop_lock(stream->core);

    if (!stream->stream || stream->flags & FFONE_STREAM_FLAG_OUTDATED_PROPS) {
        ffone_rc_ptr(RawAudioQueue) queue = stream->queue;
        bool can_update = true;

        RawAudioFormat new_format;
        can_update &= ffone_raw_audio_queue_front_buffer_format(queue, &new_format);

        uint32_t new_sample_rate;
        can_update &= ffone_raw_audio_queue_front_buffer_sample_rate(queue, &new_sample_rate);

        if (can_update) {
            ffone_pa_stream_update_props_locked(stream, new_sample_rate, new_format);
            stream->flags &= ~FFONE_STREAM_FLAG_OUTDATED_PROPS;
        }
    }

    ffone_pa_stream_try_write_locked(stream);

    ffone_pa_core_loop_unlock(stream->core);
    ffone_rc_unlock(stream);
}

struct SuccessCallbackResult {
    pa_threaded_mainloop *loop;
    int success;
};

static void ffone_pa_stream_drain_locked(ffone_rc_ptr(FFonePAStream) stream) {
    FFONE_RETURN_ON_FAILURE(stream);

    struct SuccessCallbackResult res = {
        .loop = ffone_pa_core_get_loop(stream->core),
        .success = -1,
    };

    pa_operation *o = pa_stream_drain(stream->stream, ffone_pa_stream_success_cb, &res);
    FFONE_RETURN_ON_FAILURE(o);

    if (ffone_pa_core_execute_operation(stream->core, o) == 0) {
        printf("Stream Drained: %d\n", res.success);
    }
}

static void ffone_pa_stream_success_cb(pa_stream *p, int success, void *userdata) {
    struct SuccessCallbackResult *res = userdata;

    if (res) {
        res->success = success;
        pa_threaded_mainloop_signal(res->loop, 0);
    }

    (void)p;
}

uint64_t ffone_pa_stream_get_time_locked(ffone_rc_ptr(FFonePAStream) stream) {
    FFONE_RETURN_VAL_ON_FAILURE(stream, 0);

    uint64_t time_base = stream->time_base;

    pa_usec_t usec;
    FFONE_RETURN_VAL_ON_FAILURE(
        pa_stream_get_time(stream->stream, &usec) == 0,
        time_base
    );

    return time_base + usec;
}

uint64_t ffone_pa_stream_get_time(ffone_rc_ptr(FFonePAStream) stream) {
    FFONE_RETURN_VAL_ON_FAILURE(stream, 0);

    ffone_rc_lock(stream);
    ffone_pa_core_loop_lock(stream->core);

    uint64_t time = ffone_pa_stream_get_time_locked(stream);

    ffone_pa_core_loop_unlock(stream->core);
    ffone_rc_unlock(stream);

    return time;
}
