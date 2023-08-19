#ifndef _FFONE_QUEUE_H
#define _FFONE_QUEUE_H

#include <stdlib.h>
#include <stdint.h>
#include "audio.h"

typedef struct RawAudioQueueRC RawAudioQueueRC;

struct RawAudioQueueRC *ffone_raw_audio_queue_ref(struct RawAudioQueueRC *queue);

void ffone_raw_audio_queue_unref(struct RawAudioQueueRC *queue);

int ffone_raw_audio_queue_front_buffer_format(struct RawAudioQueueRC *queue,
                                              RawAudioFormat *format);

void ffone_raw_audio_queue_read_bytes(struct RawAudioQueueRC *queue,
                                      uint8_t *bytes,
                                      size_t *nbytes,
                                      RawAudioFormat *format);

void ffone_raw_audio_queue_read_bytes_formatted(struct RawAudioQueueRC *queue,
                                                uint8_t *bytes,
                                                size_t *nbytes,
                                                RawAudioFormat format);

#endif /* _FFONE_QUEUE_H */
