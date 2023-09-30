#ifndef _FFONE_STREAM_H
#define _FFONE_STREAM_H

#include "virtual_device.h"

#include <stdint.h>

#include "audio.h"
#include "queue.h"
#include "rc.h"

#define FFONE_DEFAULT_SAMPLE_RATE 8000
#define FFONE_DEFAULT_AUDIO_FORMAT RawAudioFormat_U8

typedef uint32_t StreamFlags;

#define FFONE_STREAM_FLAG_NONE 0
#define FFONE_STREAM_FLAG_CREATED 1U << 0
#define FFONE_STREAM_FLAG_CONNECTED 1U << 1
#define FFONE_STREAM_FLAG_OUTDATED_PROPS 1U << 2

typedef struct FFonePAStream FFonePAStream;

ffone_rc(FFonePAStream) ffone_pa_stream_new(
    ffone_rc_ptr(FFonePACore) core,
    ffone_rc_ptr(FFonePAVirtualSink) sink,
    ffone_rc(RawAudioQueue) queue,
    uint32_t sample_rate,
    RawAudioFormat format
);

void ffone_pa_stream_update(ffone_rc_ptr(FFonePAStream) stream);

uint64_t ffone_pa_stream_get_time(ffone_rc_ptr(FFonePAStream) stream);

#endif /* _FFONE_STREAM_H */