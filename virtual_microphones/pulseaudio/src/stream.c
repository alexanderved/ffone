#include "stream.h"
#include "pa_ctx.h"

#include "error.h"

#include <stdio.h>
#include <string.h>

static void stream_dtor(void *opaque);

static pa_stream *new_pa_stream(
    ffone_rc_ptr(FFonePAContext) pa_ctx,
    uint32_t sample_rate,
    RawAudioFormat format
);
static int connect_pa_stream(pa_stream *stream, ffone_rc_ptr(FFonePAContext) pa_ctx);

static pa_sample_format_t raw_audio_format_to_pa_sample_format_t(RawAudioFormat raw);

static void ffone_pa_stream_update_pa_stream(ffone_rc_ptr(FFonePAStream) stream);

static void ffone_pa_stream_try_write(ffone_rc_ptr(FFonePAStream) stream);
static void ffone_pa_stream_drain(ffone_rc_ptr(FFonePAStream) stream);

static void ffone_pa_stream_success_cb(pa_stream *p, int success, void *userdata);

struct FFonePAStream {
    ffone_weak(FFonePAContext) pa_ctx;
    ffone_rc(FFonePAVirtualSink) sink;

    pa_stream *stream;
    StreamFlags flags;

    uint32_t sample_rate;
    RawAudioFormat format;
};

ffone_rc(FFonePAStream) ffone_pa_stream_new(
    ffone_rc_ptr(FFonePAContext) pa_ctx,
    ffone_rc_ptr(FFonePAVirtualSink) sink,
    uint32_t sample_rate,
    RawAudioFormat format
) {
    FFONE_RETURN_VAL_ON_FAILURE(pa_ctx && sink, NULL);

    ffone_rc(FFonePAStream) stream = ffone_rc_new0(FFonePAStream);
    FFONE_RETURN_VAL_ON_FAILURE(stream, NULL);

    FFONE_GOTO_ON_FAILURE(stream->pa_ctx = ffone_rc_ref_weak(pa_ctx), error);
    FFONE_GOTO_ON_FAILURE(stream->sink = ffone_rc_ref(sink), error);

    FFONE_GOTO_ON_FAILURE(
        stream->stream = new_pa_stream(pa_ctx, sample_rate, format),
        error
    );
    stream->flags |= FFONE_STREAM_FLAG_CREATED;

    FFONE_GOTO_ON_FAILURE(
        connect_pa_stream(stream->stream, pa_ctx) == 0,
        error
    );
    stream->flags |= FFONE_STREAM_FLAG_CONNECTED;

    stream->sample_rate = sample_rate;
    stream->format = format;

    ffone_rc_set_dtor(stream, stream_dtor);

    return stream;
error:
    if (stream->stream) {
        if (pa_stream_get_state(stream->stream) == PA_STREAM_READY) {
            pa_stream_disconnect(stream->stream);
        }
        
        pa_stream_unref(stream->stream);
    }

    if (stream->sink) ffone_rc_unref(stream->sink);
    if (stream->pa_ctx) ffone_rc_unref_weak(stream->pa_ctx);

    if (stream) ffone_rc_unref(stream);

    return NULL;
}

static void stream_dtor(void *opaque) {
    FFonePAStream *stream = opaque;
    FFONE_RETURN_ON_FAILURE(stream);

    stream->flags = FFONE_STREAM_FLAG_NONE;

    if (stream->stream) {
        ffone_pa_stream_drain(stream);
        if (pa_stream_get_state(stream->stream) == PA_STREAM_READY) {
            pa_stream_disconnect(stream->stream);
        }
        
        pa_stream_unref(stream->stream);
        stream->stream = NULL;
    }

    if (stream->sink) ffone_rc_unref(stream->sink);
    stream->sink = NULL;

    if (stream->pa_ctx) ffone_rc_unref_weak(stream->pa_ctx);
    stream->pa_ctx = NULL;
}

static pa_stream *new_pa_stream(
    ffone_rc_ptr(FFonePAContext) pa_ctx,
    uint32_t sample_rate,
    RawAudioFormat format
) {
    pa_context *context = ffone_pa_ctx_get_context(pa_ctx);
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

static int connect_pa_stream(pa_stream *stream, ffone_rc_ptr(FFonePAContext) pa_ctx) {
    FFONE_RETURN_VAL_ON_FAILURE(stream, FFONE_ERROR_INVALID_ARG);

    int ret;

    const pa_buffer_attr buf_attr = {
        .maxlength = -1,
        .tlength = -1,
        .prebuf = -1,
        .minreq = -1,
        .fragsize = -1,
    };
    pa_stream_flags_t flags = PA_STREAM_INTERPOLATE_TIMING | 
        PA_STREAM_NOT_MONOTONIC | PA_STREAM_AUTO_TIMING_UPDATE |
        PA_STREAM_ADJUST_LATENCY | PA_STREAM_VARIABLE_RATE;

    FFONE_RETURN_VAL_ON_FAILURE(
        (ret = pa_stream_connect_playback(stream, 
            /* sink->base.name */ NULL, &buf_attr, flags, NULL, NULL)) == 0,
        FFONE_ERROR(ret)
    );

    pa_stream_state_t state = PA_STREAM_UNCONNECTED;
    while (state != PA_STREAM_READY) {
        ffone_pa_ctx_iterate(pa_ctx, 1);

        state = pa_stream_get_state(stream);
        if (state == PA_STREAM_FAILED || state == PA_STREAM_TERMINATED) {
            return FFONE_ERROR_CUSTOM;
        }
    }

    return FFONE_SUCCESS;
}

static void ffone_pa_stream_update_pa_stream(ffone_rc_ptr(FFonePAStream) stream) {
    FFONE_RETURN_ON_FAILURE(stream);
    FFONE_RETURN_ON_FAILURE(!ffone_rc_is_destructed(stream));

    if (stream->stream) {
        ffone_pa_stream_drain(stream);
        if (pa_stream_get_state(stream->stream) == PA_STREAM_READY) {
            pa_stream_disconnect(stream->stream);
        }

        pa_stream_unref(stream->stream);
        stream->stream = NULL;
    }

    FFONE_RETURN_ON_FAILURE(stream->stream = new_pa_stream(
        stream->pa_ctx, stream->sample_rate, stream->format));
    FFONE_ON_FAILURE(connect_pa_stream(stream->stream, stream->pa_ctx) == 0, {
        pa_stream_unref(stream->stream);
        stream->stream = NULL;

        return;
    });
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

static void ffone_pa_stream_update_sample_rate(
    ffone_rc_ptr(FFonePAStream) stream,
    uint32_t sample_rate
) {
    FFONE_RETURN_ON_FAILURE(stream);
    FFONE_RETURN_ON_FAILURE(!ffone_rc_is_destructed(stream) && stream->stream);

    if (stream->sample_rate == sample_rate) {
        return;
    }

    ffone_pa_stream_drain(stream);

    int success = -1;
    pa_operation *o = pa_stream_update_sample_rate(
        stream->stream,
        sample_rate,
        ffone_pa_stream_success_cb,
        &success
    );
    FFONE_RETURN_ON_FAILURE(o);

    if (ffone_pa_ctx_execute_operation(stream->pa_ctx, o) == FFONE_SUCCESS) {
        printf("Stream Setting Sample Rate: %d\n", success);

        if (success) {
            stream->sample_rate = sample_rate;
        }
    }
}

static void ffone_pa_stream_update_raw_audio_format(
    ffone_rc_ptr(FFonePAStream) stream,
    RawAudioFormat format
) {
    FFONE_RETURN_ON_FAILURE(stream);
    FFONE_RETURN_ON_FAILURE(!ffone_rc_is_destructed(stream) && stream->stream);

    if (stream->format == format) {
        return;
    }

    stream->format = format;
    ffone_pa_stream_update_pa_stream(stream);
}

static void ffone_pa_stream_update_props(
    ffone_rc_ptr(FFonePAStream) stream,
    uint32_t sample_rate,
    RawAudioFormat format
) {
    FFONE_RETURN_ON_FAILURE(stream);
    FFONE_RETURN_ON_FAILURE(!ffone_rc_is_destructed(stream) && stream->stream);

    if (stream->sample_rate == sample_rate && stream->format == format) {
        return;
    }

    if (stream->sample_rate != sample_rate && stream->format == format) {
        ffone_pa_stream_update_sample_rate(stream, sample_rate);

        return;
    }

    stream->sample_rate = sample_rate;
    ffone_pa_stream_update_raw_audio_format(stream, format);
}

void ffone_pa_stream_update(ffone_rc_ptr(FFonePAStream) stream) {
    FFONE_RETURN_ON_FAILURE(stream);

    if (!stream->stream || stream->flags & FFONE_STREAM_FLAG_OUTDATED_PROPS) {
        ffone_rc_ptr(RawAudioQueue) queue = ffone_pa_ctx_get_queue(stream->pa_ctx);
        bool can_update = true;

        RawAudioFormat new_format;
        can_update &= ffone_raw_audio_queue_front_buffer_format(queue, &new_format);

        uint32_t new_sample_rate;
        can_update &= ffone_raw_audio_queue_front_buffer_sample_rate(queue, &new_sample_rate);

        if (can_update) {
            ffone_pa_stream_update_props(stream, new_sample_rate, new_format);
            stream->flags &= ~FFONE_STREAM_FLAG_OUTDATED_PROPS;
        }
    }

    ffone_pa_stream_try_write(stream);
}

static void ffone_pa_stream_try_write(ffone_rc_ptr(FFonePAStream) stream) {
    FFONE_RETURN_ON_FAILURE(stream);
    FFONE_RETURN_ON_FAILURE(!ffone_rc_is_destructed(stream) && stream->stream);

    RawAudioQueue *queue = ffone_pa_ctx_get_queue(stream->pa_ctx);
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

    while (write_buffer_cursor < write_buffer_end && ffone_raw_audio_queue_has_bytes(queue)) {
        size_t read_size = write_buffer_end - write_buffer_cursor;
        bool have_same_props = false;
        ffone_raw_audio_queue_read_bytes_with_props(
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

static void ffone_pa_stream_drain(ffone_rc_ptr(FFonePAStream) stream) {
    FFONE_RETURN_ON_FAILURE(stream);
    FFONE_RETURN_ON_FAILURE(!ffone_rc_is_destructed(stream) && stream->stream);

    int success = -1;
    pa_operation *o = pa_stream_drain(stream->stream, ffone_pa_stream_success_cb, &success);
    FFONE_RETURN_ON_FAILURE(o);

    if (ffone_pa_ctx_execute_operation(stream->pa_ctx, o) == 0) {
        printf("Stream Drained: %d\n", success);
    }
}

static void ffone_pa_stream_success_cb(pa_stream *p, int success, void *userdata) {
    int *success_ret = userdata;
    FFONE_RETURN_ON_FAILURE(success_ret);

    *success_ret = success;

    (void)p;
}

uint64_t ffone_pa_stream_get_time(ffone_rc_ptr(FFonePAStream) stream) {
    FFONE_RETURN_VAL_ON_FAILURE(stream, 0);
    FFONE_RETURN_VAL_ON_FAILURE(!ffone_rc_is_destructed(stream) && stream->stream, 0);

    pa_usec_t usec;
    pa_stream_get_time(stream->stream, &usec);

    return usec;
}