#ifndef _FFONE_AUDIO_H
#define _FFONE_AUDIO_H

#include <stdint.h>
#include <stddef.h>

enum RawAudioFormat {
    RawAudioFormat_U8,
    RawAudioFormat_S16LE,
    RawAudioFormat_S16BE,
    RawAudioFormat_S24LE,
    RawAudioFormat_S24BE,
    RawAudioFormat_S32LE,
    RawAudioFormat_S32BE,
    RawAudioFormat_F32LE,
    RawAudioFormat_F32BE,
    RawAudioFormat_Unspecified,
};
typedef int8_t RawAudioFormat;

typedef struct RawAudioBuffer RawAudioBuffer;

void ffone_raw_audio_buffer_drop(void *buffer);

const uint8_t *ffone_raw_audio_buffer_as_ptr(const struct RawAudioBuffer *buffer);

uint8_t *ffone_raw_audio_buffer_as_ptr_mut(struct RawAudioBuffer *buffer);

int ffone_raw_audio_buffer_format(const struct RawAudioBuffer *buffer, RawAudioFormat *format);

size_t ffone_raw_audio_buffer_len(const struct RawAudioBuffer *buffer);

size_t ffone_raw_audio_buffer_no_samples(const struct RawAudioBuffer *buffer);

#endif /* _FFONE_AUDIO_H */
