#ifndef _FFONE_QUEUE_H
#define _FFONE_QUEUE_H

#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include "audio.h"

typedef struct RawAudioQueue RawAudioQueue;

RawAudioQueue *ffone_raw_audio_queue_new(void);

bool ffone_raw_audio_queue_has_bytes(RawAudioQueue *queue);

bool ffone_raw_audio_queue_has_buffers(RawAudioQueue *queue);

bool ffone_raw_audio_queue_front_buffer_format(RawAudioQueue *queue, RawAudioFormat *format);

RawAudioBuffer *ffone_raw_audio_queue_pop_buffer(RawAudioQueue *queue);

RawAudioBuffer *ffone_raw_audio_queue_pop_buffer_formatted(RawAudioQueue *queue,
                                                           RawAudioFormat format,
                                                           bool *have_same_format);

void ffone_raw_audio_queue_read_bytes(RawAudioQueue *queue,
                                      uint8_t *bytes,
                                      size_t *nbytes,
                                      RawAudioFormat *format);

void ffone_raw_audio_queue_read_bytes_formatted(RawAudioQueue *queue,
                                                uint8_t *bytes,
                                                size_t *nbytes,
                                                RawAudioFormat format,
                                                bool *have_same_format);

#endif /* _FFONE_QUEUE_H */
