#ifndef _FFONE_STREAM_H
#define _FFONE_STREAM_H

#include "virtual_device.h"

#include <stdint.h>

#include "audio.h"
#include "rc.h"

#define FFONE_DEFAULT_SAMPLE_RATE 8000
#define FFONE_DEFAULT_AUDIO_FORMAT RawAudioFormat_U8

typedef uint32_t StreamFlags;

#define FFONE_STREAM_FLAG_NONE 0
#define FFONE_STREAM_FLAG_CREATED 1U << 0
#define FFONE_STREAM_FLAG_CONNECTED 1U << 1
#define FFONE_STREAM_FLAG_OUTDATED_PROPS 1U << 2

typedef struct Stream Stream;

Stream *stream_new(
    ffone_rc_ptr(PAContext) pa_ctx,
    ffone_rc_ptr(VirtualSink) sink,
    uint32_t sample_rate,
    RawAudioFormat format
);

void stream_update(Stream *stream);

uint64_t stream_get_time(Stream *stream);

#endif /* _FFONE_STREAM_H */