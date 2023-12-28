#ifndef _FFONE_QUEUE_H
#define _FFONE_QUEUE_H

#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include "audio.h"

typedef struct RawAudioQueue RawAudioQueue;

RawAudioQueue *ffone_raw_audio_queue_new(uint64_t max_duration);

bool ffone_raw_audio_queue_has_bytes_locked(RawAudioQueue *queue);

bool ffone_raw_audio_queue_has_bytes(RawAudioQueue *queue);

bool ffone_raw_audio_queue_has_buffers(RawAudioQueue *queue);

bool ffone_raw_audio_queue_front_buffer_format(RawAudioQueue *queue, RawAudioFormat *format);

bool ffone_raw_audio_queue_front_buffer_sample_rate(RawAudioQueue *queue, uint32_t *sample_rate);

void ffone_raw_audio_queue_read_bytes_locked(RawAudioQueue *queue,
                                             uint8_t *bytes,
                                             size_t *nbytes,
                                             RawAudioFormat *format,
                                             uint32_t *sample_rate);

void ffone_raw_audio_queue_read_bytes(RawAudioQueue *queue,
                                      uint8_t *bytes,
                                      size_t *nbytes,
                                      RawAudioFormat *format,
                                      uint32_t *sample_rate);

void ffone_raw_audio_queue_read_bytes_with_props_locked(RawAudioQueue *queue,
                                                        uint8_t *bytes,
                                                        size_t *nbytes,
                                                        RawAudioFormat format,
                                                        uint32_t sample_rate,
                                                        bool *have_same_props);

void ffone_raw_audio_queue_read_bytes_with_props(RawAudioQueue *queue,
                                                 uint8_t *bytes,
                                                 size_t *nbytes,
                                                 RawAudioFormat format,
                                                 uint32_t sample_rate,
                                                 bool *have_same_props);

#endif /* _FFONE_QUEUE_H */
