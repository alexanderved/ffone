#ifndef _FFONE_VIRTUAL_DEVICE_H
#define _FFONE_VIRTUAL_DEVICE_H

#include <stdint.h>

#include "audio.h"
#include "rc.h"

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

ffone_rc(VirtualSource) virtual_source_new(
    ffone_rc_ptr(PAContext) pa_ctx,
    ffone_rc_ptr(VirtualSink) master
);

ffone_rc(VirtualSink) virtual_sink_new(ffone_rc_ptr(PAContext) pa_ctx);
const char *virtual_sink_get_name(ffone_rc_ptr(VirtualSink) sink);

#endif /* _FFONE_VIRTUAL_DEVICE_H */