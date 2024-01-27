#ifndef _FFONE_VIRTUAL_DEVICE_H
#define _FFONE_VIRTUAL_DEVICE_H

#include "core.h"

#include "audio.h"
#include "rc.h"

#include <stdint.h>

#include <pulse/pulseaudio.h>

#define FFONE_PA_DEFAULT_SAMPLE_RATE 48000
#define FFONE_PA_VIRTUAL_DEVICE_INDEX_NONE UINT32_MAX

typedef struct FFonePAVirtualSource FFonePAVirtualSource;
typedef struct FFonePAVirtualSink FFonePAVirtualSink;

typedef uint32_t FFonePAVirtualDeviceFlags;

#define FFONE_PA_VIRTUAL_DEVICE_FLAGS_NONE 0
#define FFONE_PA_VIRTUAL_DEVICE_FLAGS_CREATED (1U << 0)
#define FFONE_PA_VIRTUAL_DEVICE_FLAGS_LOADED (1U << 1)

ffone_rc(FFonePAVirtualSource) ffone_pa_virtual_source_new(
    FFonePACore *core,
    FFonePAVirtualSink *master
);

ffone_rc(FFonePAVirtualSink) ffone_pa_virtual_sink_new(FFonePACore *core);

#endif /* _FFONE_VIRTUAL_DEVICE_H */