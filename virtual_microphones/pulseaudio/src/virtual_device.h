#ifndef _FFONE_VIRTUAL_DEVICE_H
#define _FFONE_VIRTUAL_DEVICE_H

#include <stdint.h>

#include "audio.h"

#include <pulse/pulseaudio.h>

#define DEFAULT_SAMPLE_RATE 48000
#define VIRTUAL_DEVICE_INDEX_NONE UINT32_MAX

typedef struct PAContext PAContext;

typedef struct VirtualDevice VirtualDevice;
typedef struct VirtualSource VirtualSource;
typedef struct VirtualSink VirtualSink;

typedef uint32_t VirtualDeviceFlags;

#define VIRTUAL_DEVICE_FLAGS_NONE 0
#define VIRTUAL_DEVICE_FLAGS_CREATED 1U << 0
#define VIRTUAL_DEVICE_FLAGS_LOADED 1U << 1

VirtualSource *virtual_source_new(PAContext *pa_ctx, VirtualSink *master);

VirtualSink *virtual_sink_new(PAContext *pa_ctx);
const char *virtual_sink_get_name(VirtualSink *sink);

#endif /* _FFONE_VIRTUAL_DEVICE_H */