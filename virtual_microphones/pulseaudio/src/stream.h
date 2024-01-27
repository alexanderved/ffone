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
#define FFONE_STREAM_FLAG_PLAYING (1U << 1)
#define FFONE_STREAM_FLAG_OUTDATED_PROPS (1U << 2)
#define FFONE_STREAM_FLAG_DESTRUCTING (1U << 3)

typedef struct FFonePAStream FFonePAStream;

ffone_rc(FFonePAStream) ffone_pa_stream_new(
    FFonePACore *core,
    RawAudioQueue *queue
);

void ffone_pa_stream_play(FFonePAStream *stream);

uint64_t ffone_pa_stream_get_time(FFonePAStream *stream);

#endif /* _FFONE_STREAM_H */