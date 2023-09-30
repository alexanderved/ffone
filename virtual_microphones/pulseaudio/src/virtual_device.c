#include "virtual_device.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "util.h"
#include "error.h"

static const char *ffone_pa_virtual_sink_get_name(ffone_rc_ptr(FFonePAVirtualSink) sink);

typedef struct FFonePAVirtualDevice {
    ffone_rc(FFonePACore) core;
    FFonePAVirtualDeviceFlags flags;

    uint32_t idx;
    char *name;
    char *descr;
} FFonePAVirtualDevice;

static int virtual_device_new(
    FFonePAVirtualDevice *device,
    ffone_rc_ptr(FFonePACore) core,
    const char *name,
    const char *descr
) {
    FFONE_RETURN_VAL_ON_FAILURE(device && core && name && descr, FFONE_ERROR_INVALID_ARG);

    int ret;

    FFONE_RETURN_VAL_ON_FAILURE(
        device->core = ffone_rc_ref(core),
        FFONE_ERROR_CUSTOM
    );

    device->flags = FFONE_PA_VIRTUAL_DEVICE_FLAGS_CREATED;
    device->idx = FFONE_PA_VIRTUAL_DEVICE_INDEX_NONE;
    device->name = NULL;
    device->descr = NULL;

    int did = rand();
    int pid = ffone_get_pid();
    size_t device_name_len = snprintf(NULL, 0, "%d-%s-%d", did, name, pid);

    ret = FFONE_ERROR_BAD_ALLOC;
    FFONE_GOTO_ON_FAILURE(device->name = malloc(device_name_len + 1), error);
    sprintf(device->name, "%d-%s-%d", did, name, pid);

    printf("NAME: %s\n", device->name);

    ret = FFONE_ERROR_BAD_ALLOC;
    FFONE_GOTO_ON_FAILURE(device->descr = malloc(strlen(descr) + 1), error);
    strcpy(device->descr, descr);

    return FFONE_SUCCESS;
error:
    if (device->descr) free(device->descr);
    if (device->name) free(device->name);
    if (device->core) ffone_rc_unref(device->core);

    return ret;
}

static void virtual_device_delete(FFonePAVirtualDevice *device) {
    FFONE_RETURN_ON_FAILURE(device);

    if (device->descr) free(device->descr);
    device->descr = NULL;

    if (device->name) free(device->name);
    device->name = NULL;

    device->idx = FFONE_PA_VIRTUAL_DEVICE_INDEX_NONE;
    device->flags = FFONE_PA_VIRTUAL_DEVICE_FLAGS_NONE;

    if (device->core) ffone_rc_unref(device->core);
    device->core = NULL;
}

static void virtual_device_loaded(pa_context *c, uint32_t idx, void *userdata) {
    FFonePAVirtualDevice *device = (FFonePAVirtualDevice *)userdata;
    printf("Virtual Device Index: %u\n", idx);

    if (device) {
        device->flags |= FFONE_PA_VIRTUAL_DEVICE_FLAGS_LOADED;
        device->idx = idx;
    }

    (void)c;
}

static void virtual_device_unloaded(pa_context *c, int success, void *userdata) {
    FFonePAVirtualDevice *device = (FFonePAVirtualDevice *)userdata;
    printf("Virtual Device Unloaded: %d\n", success);

    if (success && device) {
        device->flags &= ~FFONE_PA_VIRTUAL_DEVICE_FLAGS_LOADED;
        device->idx = FFONE_PA_VIRTUAL_DEVICE_INDEX_NONE;
    }

    (void)c;
}

struct FFonePAVirtualSource {
    FFonePAVirtualDevice base;
    ffone_rc(FFonePAVirtualSink) master;
};

static void ffone_pa_virtual_source_dtor(void *opaque);
static int ffone_pa_virtual_source_load(ffone_rc_ptr(FFonePAVirtualSource) src);
static int ffone_pa_virtual_source_unload(ffone_rc_ptr(FFonePAVirtualSource) src);

ffone_rc(FFonePAVirtualSource) ffone_pa_virtual_source_new(
    ffone_rc_ptr(FFonePACore) core,
    ffone_rc_ptr(FFonePAVirtualSink) master)
{
    FFONE_RETURN_VAL_ON_FAILURE(core && master, NULL);

    ffone_rc(FFonePAVirtualSource) src = ffone_rc_new0(FFonePAVirtualSource);
    FFONE_RETURN_VAL_ON_FAILURE(src, NULL);

    FFONE_GOTO_ON_FAILURE(src->master = ffone_rc_ref(master), error_master_rc_ref);
    FFONE_GOTO_ON_FAILURE(virtual_device_new(
        &src->base,
        core,
        "ffone_pa_virtual_source",
        "FFone_Virtual_Microphone"
    ) == 0, error_virtual_device_new);
    FFONE_GOTO_ON_FAILURE(ffone_pa_virtual_source_load(src) == 0, error_virtual_source_load);

    ffone_rc_set_dtor(src, ffone_pa_virtual_source_dtor);

    return src;

error_virtual_source_load:
    virtual_device_delete(&src->base);
error_virtual_device_new:
    ffone_rc_unref(src->master);
error_master_rc_ref:
    ffone_rc_unref(src);

    return NULL;
}

static void ffone_pa_virtual_source_dtor(void *opaque) {
    FFonePAVirtualSource *src = opaque;
    FFONE_RETURN_ON_FAILURE(src);

    ffone_pa_virtual_source_unload(src);
    ffone_rc_unref(src->master);
    src->master = NULL;
    virtual_device_delete(&src->base);

    puts("FFonePAVirtualSource dtor");
}

static int ffone_pa_virtual_source_load(ffone_rc_ptr(FFonePAVirtualSource) src) {
    FFONE_RETURN_VAL_ON_FAILURE(src, FFONE_ERROR_INVALID_ARG);

    FFONE_RETURN_VAL_ON_FAILURE(src->master, FFONE_ERROR_BAD_STATE);
    FFONE_RETURN_VAL_ON_FAILURE(
        (src->base.flags & FFONE_PA_VIRTUAL_DEVICE_FLAGS_CREATED)
            && !(src->base.flags & FFONE_PA_VIRTUAL_DEVICE_FLAGS_LOADED),
        FFONE_ERROR_BAD_STATE
    );

    char *args = ffone_format_str(
        "source_name=%s source_properties=device.description=%s master=%s.monitor "
        "master_channel_map=%s rate=%d channels=%d channel_map=%s",
        src->base.name,
        src->base.descr,
        ffone_pa_virtual_sink_get_name(src->master),
        "mono",
        FFONE_PA_DEFAULT_SAMPLE_RATE,
        1,
        "mono"
    );
    FFONE_RETURN_VAL_ON_FAILURE(args, FFONE_ERROR_BAD_ALLOC);

    int ret = ffone_pa_core_load_virtual_device(
        src->base.core,
        "module-remap-source",
        args,
        virtual_device_loaded,
        &src->base
    );

    free(args);

    return FFONE_ERROR(ret);
}

static int ffone_pa_virtual_source_unload(ffone_rc_ptr(FFonePAVirtualSource) src) {
    FFONE_RETURN_VAL_ON_FAILURE(src, FFONE_ERROR_INVALID_ARG);
    
    FFONE_RETURN_VAL_ON_FAILURE((src->base.flags & FFONE_PA_VIRTUAL_DEVICE_FLAGS_CREATED)
        && (src->base.flags & FFONE_PA_VIRTUAL_DEVICE_FLAGS_LOADED), FFONE_ERROR_BAD_STATE);

    int ret = ffone_pa_core_unload_virtual_device(
        src->base.core,
        src->base.idx,
        virtual_device_unloaded,
        &src->base
    );

    return FFONE_ERROR(ret);
}

struct FFonePAVirtualSink {
    FFonePAVirtualDevice base;
};

static void ffone_pa_virtual_sink_dtor(void *opaque);
static int ffone_pa_virtual_sink_load(ffone_rc_ptr(FFonePAVirtualSink) sink);
static int ffone_pa_virtual_sink_unload(ffone_rc_ptr(FFonePAVirtualSink) sink);

ffone_rc(FFonePAVirtualSink) ffone_pa_virtual_sink_new(ffone_rc_ptr(FFonePACore) core) {
    FFONE_RETURN_VAL_ON_FAILURE(core, NULL);

    ffone_rc(FFonePAVirtualSink) sink = ffone_rc_new0(FFonePAVirtualSink);
    FFONE_RETURN_VAL_ON_FAILURE(sink, NULL);

    FFONE_GOTO_ON_FAILURE(virtual_device_new(
        &sink->base,
        core,
        "ffone_pa_virtual_sink",
        "FFone_Output"
    ) == 0, error_virtual_device_new);
    FFONE_GOTO_ON_FAILURE(ffone_pa_virtual_sink_load(sink) == 0, error_virtual_sink_load);

    ffone_rc_set_dtor(sink, ffone_pa_virtual_sink_dtor);

    return sink;

error_virtual_sink_load:
    virtual_device_delete(&sink->base);
error_virtual_device_new:
    ffone_rc_unref(sink);

    return NULL;
}

static void ffone_pa_virtual_sink_dtor(void *opaque) {
    FFonePAVirtualSink *sink = opaque;
    FFONE_RETURN_ON_FAILURE(sink);

    ffone_pa_virtual_sink_unload(sink);
    virtual_device_delete(&sink->base);

    puts("FFonePAVirtualSink dtor");
}

static int ffone_pa_virtual_sink_load(ffone_rc_ptr(FFonePAVirtualSink) sink) {
    FFONE_RETURN_VAL_ON_FAILURE(sink, FFONE_ERROR_INVALID_ARG);

    FFONE_RETURN_VAL_ON_FAILURE(
        (sink->base.flags & FFONE_PA_VIRTUAL_DEVICE_FLAGS_CREATED)
            && !(sink->base.flags & FFONE_PA_VIRTUAL_DEVICE_FLAGS_LOADED),
        FFONE_ERROR_BAD_STATE
    );

    char *args = ffone_format_str(
        "sink_name=%s sink_properties=device.description=%s "
        "rate=%d channels=%d channel_map=%s",
        sink->base.name,
        sink->base.descr,
        FFONE_PA_DEFAULT_SAMPLE_RATE,
        1,
        "mono"
    );
    FFONE_RETURN_VAL_ON_FAILURE(args, FFONE_ERROR_BAD_ALLOC);

    int ret = ffone_pa_core_load_virtual_device(
        sink->base.core,
        "module-null-sink",
        args,
        virtual_device_loaded,
        &sink->base
    );

    free(args);

    return FFONE_ERROR(ret);
}

static int ffone_pa_virtual_sink_unload(ffone_rc_ptr(FFonePAVirtualSink) sink) {
    FFONE_RETURN_VAL_ON_FAILURE(sink, FFONE_ERROR_INVALID_ARG);

    FFONE_RETURN_VAL_ON_FAILURE(
        (sink->base.flags & FFONE_PA_VIRTUAL_DEVICE_FLAGS_CREATED)
            && (sink->base.flags & FFONE_PA_VIRTUAL_DEVICE_FLAGS_LOADED),
        FFONE_ERROR_BAD_STATE
    );

    int ret = ffone_pa_core_unload_virtual_device(
        sink->base.core,
        sink->base.idx,
        virtual_device_unloaded,
        &sink->base
    );

    return FFONE_ERROR(ret);
}

static const char *ffone_pa_virtual_sink_get_name(ffone_rc_ptr(FFonePAVirtualSink) sink) {
    FFONE_RETURN_VAL_ON_FAILURE(sink, NULL);

    return sink->base.name;
}