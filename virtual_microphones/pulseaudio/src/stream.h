#ifndef _FFONE_STREAM_H
#define _FFONE_STREAM_H

#include "virtual_device.h"

#include <stdint.h>

#include "audio.h"

#define FFONE_DEFAULT_SAMPLE_RATE 8000
#define FFONE_DEFAULT_AUDIO_FORMAT RawAudioFormat_U8

typedef uint32_t StreamFlags;

#define FFONE_STREAM_FLAG_NONE 0
#define FFONE_STREAM_FLAG_CREATED 1U << 0
#define FFONE_STREAM_FLAG_CONNECTED 1U << 1
#define FFONE_STREAM_FLAG_OUTDATED_AUDIO_FORMAT 1U << 2

typedef struct Stream Stream;

Stream *stream_new(
    PAContext *pa_ctx,
    VirtualSink *sink,
    uint32_t sample_rate,
    RawAudioFormat format
);

void stream_update(Stream *stream);



pa_stream *stream_get_pa_stream(Stream *s);

#endif /* _FFONE_STREAM_H */