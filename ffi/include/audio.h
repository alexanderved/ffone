#ifndef _FFONE_AUDIO_H
#define _FFONE_AUDIO_H

#include <stdint.h>

enum RawAudioFormat {
    U8,
    S16LE,
    S16BE,
    S24LE,
    S24BE,
    S32LE,
    S32BE,
    F32LE,
    F32BE,
};
typedef int8_t RawAudioFormat;

#endif /* _FFONE_AUDIO_H */
