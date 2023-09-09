#include "pa_ctx.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "util.h"
#include "error.h"

struct VirtualDevice {
    ffone_weak(PAContext) pa_ctx;
    VirtualDeviceFlags flags;

    uint32_t idx;
    char *name;
    char *descr;
};

static int virtual_device_new(
    VirtualDevice *device,
    ffone_rc_ptr(PAContext) pa_ctx,
    const char *name,
    const char *descr
) {
    FFONE_RETURN_VAL_ON_FAILURE(device && pa_ctx && name && descr, FFONE_ERROR_INVALID_ARG);

    int ret;

    FFONE_RETURN_VAL_ON_FAILURE(
        device->pa_ctx = ffone_rc_ref_weak(pa_ctx),
        FFONE_ERROR_CUSTOM
    );

    device->flags = VIRTUAL_DEVICE_FLAGS_CREATED;
    device->idx = VIRTUAL_DEVICE_INDEX_NONE;
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
    if (device->pa_ctx) ffone_rc_unref_weak(device->pa_ctx);

    return ret;
}

static void virtual_device_delete(VirtualDevice *device) {
    FFONE_RETURN_ON_FAILURE(device);

    if (device->descr) free(device->descr);
    device->descr = NULL;

    if (device->name) free(device->name);
    device->name = NULL;

    device->idx = VIRTUAL_DEVICE_INDEX_NONE;
    device->flags = VIRTUAL_DEVICE_FLAGS_NONE;

    if (device->pa_ctx) ffone_rc_unref_weak(device->pa_ctx);
    device->pa_ctx = NULL;
}

static void virtual_device_loaded(pa_context *c, uint32_t idx, void *userdata) {
    VirtualDevice *device = (VirtualDevice *)userdata;
    printf("Virtual Device Index: %u\n", idx);

    if (device) {
        device->flags |= VIRTUAL_DEVICE_FLAGS_LOADED;
        device->idx = idx;
    }

    (void)c;
}

static void virtual_device_unloaded(pa_context *c, int success, void *userdata) {
    VirtualDevice *device = (VirtualDevice *)userdata;
    printf("Virtual Device Unloaded: %d\n", success);

    if (success && device) {
        device->flags &= ~VIRTUAL_DEVICE_FLAGS_LOADED;
        device->idx = VIRTUAL_DEVICE_INDEX_NONE;
    }

    (void)c;
}

struct VirtualSource {
    VirtualDevice base;
    ffone_rc(VirtualSink) master;
};

static void virtual_source_dtor(void *opaque);
static int virtual_source_load(ffone_rc_ptr(VirtualSource) src);
static int virtual_source_unload(ffone_rc_ptr(VirtualSource) src);

ffone_rc(VirtualSource) virtual_source_new(
    ffone_rc_ptr(PAContext) pa_ctx,
    ffone_rc_ptr(VirtualSink) master)
{
    FFONE_RETURN_VAL_ON_FAILURE(pa_ctx && master, NULL);

    ffone_rc(VirtualSource) src = ffone_rc_new0(VirtualSource);
    FFONE_RETURN_VAL_ON_FAILURE(src, NULL);

    FFONE_GOTO_ON_FAILURE(src->master = ffone_rc_ref(master), error_master_rc_ref);
    FFONE_GOTO_ON_FAILURE(virtual_device_new(
        &src->base,
        pa_ctx,
        "ffone_pa_virtual_source",
        "FFone_Virtual_Microphone"
    ) == 0, error_virtual_device_new);
    FFONE_GOTO_ON_FAILURE(virtual_source_load(src) == 0, error_virtual_source_load);

    ffone_rc_set_dtor(src, virtual_source_dtor);

    return src;

error_virtual_source_load:
    virtual_device_delete(&src->base);
error_virtual_device_new:
    ffone_rc_unref(src->master);
error_master_rc_ref:
    ffone_rc_unref(src);

    return NULL;
}

static void virtual_source_dtor(void *opaque) {
    VirtualSource *src = opaque;
    FFONE_RETURN_ON_FAILURE(src);

    virtual_source_unload(src);
    ffone_rc_unref(src->master);
    src->master = NULL;
    virtual_device_delete(&src->base);

    puts("VirtualSource dtor");
}

static int virtual_source_load(ffone_rc_ptr(VirtualSource) src) {
    FFONE_RETURN_VAL_ON_FAILURE(src, FFONE_ERROR_INVALID_ARG);

    FFONE_RETURN_VAL_ON_FAILURE(src->master, FFONE_ERROR_BAD_STATE);
    FFONE_RETURN_VAL_ON_FAILURE(
        (src->base.flags & VIRTUAL_DEVICE_FLAGS_CREATED)
            && !(src->base.flags & VIRTUAL_DEVICE_FLAGS_LOADED),
        FFONE_ERROR_BAD_STATE
    );

    char *args = ffone_format_str(
        "source_name=%s source_properties=device.description=%s master=%s.monitor "
        "master_channel_map=%s rate=%d channels=%d channel_map=%s",
        src->base.name,
        src->base.descr,
        virtual_sink_get_name(src->master),
        "mono",
        DEFAULT_SAMPLE_RATE,
        1,
        "mono"
    );
    FFONE_RETURN_VAL_ON_FAILURE(args, FFONE_ERROR_BAD_ALLOC);

    int ret = pa_ctx_load_virtual_device(
        src->base.pa_ctx,
        "module-remap-source",
        args,
        virtual_device_loaded,
        &src->base
    );

    free(args);

    return FFONE_ERROR(ret);
}

static int virtual_source_unload(ffone_rc_ptr(VirtualSource) src) {
    FFONE_RETURN_VAL_ON_FAILURE(src, FFONE_ERROR_INVALID_ARG);
    
    FFONE_RETURN_VAL_ON_FAILURE((src->base.flags & VIRTUAL_DEVICE_FLAGS_CREATED)
        && (src->base.flags & VIRTUAL_DEVICE_FLAGS_LOADED), FFONE_ERROR_BAD_STATE);

    int ret = pa_ctx_unload_virtual_device(
        src->base.pa_ctx,
        src->base.idx,
        virtual_device_unloaded,
        &src->base
    );

    return FFONE_ERROR(ret);
}

struct VirtualSink {
    VirtualDevice base;
};

static void virtual_sink_dtor(void *opaque);
static int virtual_sink_load(ffone_rc_ptr(VirtualSink) sink);
static int virtual_sink_unload(ffone_rc_ptr(VirtualSink) sink);

ffone_rc(VirtualSink) virtual_sink_new(ffone_rc_ptr(PAContext) pa_ctx) {
    FFONE_RETURN_VAL_ON_FAILURE(pa_ctx, NULL);

    ffone_rc(VirtualSink) sink = ffone_rc_new0(VirtualSink);
    FFONE_RETURN_VAL_ON_FAILURE(sink, NULL);

    FFONE_GOTO_ON_FAILURE(virtual_device_new(
        &sink->base,
        pa_ctx,
        "ffone_pa_virtual_sink",
        "FFone_Output"
    ) == 0, error_virtual_device_new);
    FFONE_GOTO_ON_FAILURE(virtual_sink_load(sink) == 0, error_virtual_sink_load);

    ffone_rc_set_dtor(sink, virtual_sink_dtor);

    return sink;

error_virtual_sink_load:
    virtual_device_delete(&sink->base);
error_virtual_device_new:
    ffone_rc_unref(sink);

    return NULL;
}

static void virtual_sink_dtor(void *opaque) {
    VirtualSink *sink = opaque;
    FFONE_RETURN_ON_FAILURE(sink);

    virtual_sink_unload(sink);
    virtual_device_delete(&sink->base);

    puts("VirtualSink dtor");
}

int virtual_sink_load(ffone_rc_ptr(VirtualSink) sink) {
    FFONE_RETURN_VAL_ON_FAILURE(sink, FFONE_ERROR_INVALID_ARG);

    FFONE_RETURN_VAL_ON_FAILURE(
        (sink->base.flags & VIRTUAL_DEVICE_FLAGS_CREATED)
            && !(sink->base.flags & VIRTUAL_DEVICE_FLAGS_LOADED),
        FFONE_ERROR_BAD_STATE
    );

    char *args = ffone_format_str(
        "sink_name=%s sink_properties=device.description=%s "
        "rate=%d channels=%d channel_map=%s",
        sink->base.name,
        sink->base.descr,
        DEFAULT_SAMPLE_RATE,
        1,
        "mono"
    );
    FFONE_RETURN_VAL_ON_FAILURE(args, FFONE_ERROR_BAD_ALLOC);

    int ret = pa_ctx_load_virtual_device(
        sink->base.pa_ctx,
        "module-null-sink",
        args,
        virtual_device_loaded,
        &sink->base
    );

    free(args);

    return FFONE_ERROR(ret);
}

int virtual_sink_unload(ffone_rc_ptr(VirtualSink) sink) {
    FFONE_RETURN_VAL_ON_FAILURE(sink, FFONE_ERROR_INVALID_ARG);

    FFONE_RETURN_VAL_ON_FAILURE(
        (sink->base.flags & VIRTUAL_DEVICE_FLAGS_CREATED)
            && (sink->base.flags & VIRTUAL_DEVICE_FLAGS_LOADED),
        FFONE_ERROR_BAD_STATE
    );

    int ret = pa_ctx_unload_virtual_device(
        sink->base.pa_ctx,
        sink->base.idx,
        virtual_device_unloaded,
        &sink->base
    );

    return FFONE_ERROR(ret);
}

const char *virtual_sink_get_name(ffone_rc_ptr(VirtualSink) sink) {
    FFONE_RETURN_VAL_ON_FAILURE(sink, NULL);

    return sink->base.name;
}