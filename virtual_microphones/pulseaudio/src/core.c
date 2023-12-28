#include "core.h"
#include "virtual_device.h"

#include "error.h"

#include <stdio.h>
#include <stdlib.h>

struct FFonePACore {
    pa_threaded_mainloop *loop; /* const */
    pa_context *context; /* const */
};

static void ffone_pa_core_dtor(void *opaque);

static void context_state_cb(pa_context *context, void *userdata)
{
    pa_threaded_mainloop *loop = userdata;

    switch (pa_context_get_state(context)) {
        case PA_CONTEXT_READY:
        case PA_CONTEXT_FAILED:
        case PA_CONTEXT_TERMINATED:
            pa_threaded_mainloop_signal(loop, 0);
        default:
            break;
    }
}

static int connect_pa_context(pa_context *context, pa_threaded_mainloop *loop) {
    int ret = 0;

    pa_context_set_state_callback(context, context_state_cb, loop);

    ret = pa_context_connect(context, NULL, PA_CONTEXT_NOAUTOSPAWN, NULL);
    if (ret < 0) {
        return ret;
    }

    pa_context_state_t state = PA_CONTEXT_UNCONNECTED;
    while ((state = pa_context_get_state(context)) != PA_CONTEXT_READY) {
        if (state == PA_CONTEXT_FAILED || state == PA_CONTEXT_TERMINATED)
            return -1;

        pa_threaded_mainloop_wait(loop);
    }

    pa_context_set_state_callback(context, NULL, NULL);

    return 0;
}

ffone_rc(FFonePACore) ffone_pa_core_new(void) {
    ffone_rc(FFonePACore) core = ffone_rc_new0(FFonePACore);
    FFONE_RETURN_VAL_ON_FAILURE(core, NULL);

    FFONE_GOTO_ON_FAILURE(core->loop = pa_threaded_mainloop_new(), mainloop_new_error);
    FFONE_GOTO_ON_FAILURE(pa_threaded_mainloop_start(core->loop) == 0, mainloop_start_error);

    pa_threaded_mainloop_lock(core->loop);

    pa_mainloop_api *api = pa_threaded_mainloop_get_api(core->loop);
    FFONE_GOTO_ON_FAILURE(api, mainloop_get_api_error);

    FFONE_GOTO_ON_FAILURE(
        core->context = pa_context_new(api, "ffone_pa_virtual_microphone"),
        context_new_error
    );
    
    FFONE_GOTO_ON_FAILURE(
        connect_pa_context(core->context, core->loop) == 0,
        context_connect_error
    );

    ffone_rc_set_dtor(core, ffone_pa_core_dtor);

    pa_threaded_mainloop_unlock(core->loop);

    return core;
context_connect_error:
    pa_context_unref(core->context);
context_new_error:
mainloop_get_api_error:
    pa_threaded_mainloop_unlock(core->loop);
    pa_threaded_mainloop_stop(core->loop);
mainloop_start_error:
    pa_threaded_mainloop_free(core->loop);
mainloop_new_error:
    if (core) ffone_rc_unref(core);

    return NULL;
}

static void ffone_pa_core_dtor(void *opaque) {
    FFonePACore *core = opaque;
    FFONE_RETURN_ON_FAILURE(core);

    pa_threaded_mainloop_lock(core->loop);

    if (core->context) {
        pa_context_disconnect(core->context);
        pa_context_unref(core->context);
        core->context = NULL;
    }

    pa_threaded_mainloop_unlock(core->loop);
    pa_threaded_mainloop_stop(core->loop);
    pa_threaded_mainloop_free(core->loop);
    core->loop = NULL;
}

pa_context *ffone_pa_core_get_context(ffone_rc_ptr(FFonePACore) core) {
    FFONE_RETURN_VAL_ON_FAILURE(core, NULL);

    return core->context;
}

pa_threaded_mainloop *ffone_pa_core_get_loop(ffone_rc_ptr(FFonePACore) core) {
    FFONE_RETURN_VAL_ON_FAILURE(core, NULL);

    return core->loop;
}

void ffone_pa_core_loop_lock(ffone_rc_ptr(FFonePACore) core) {
    FFONE_RETURN_ON_FAILURE(core);

    pa_threaded_mainloop_lock(ffone_pa_core_get_loop(core));
}

void ffone_pa_core_loop_unlock(ffone_rc_ptr(FFonePACore) core) {
    FFONE_RETURN_ON_FAILURE(core);

    pa_threaded_mainloop_unlock(ffone_pa_core_get_loop(core));
}

void ffone_pa_core_loop_signal(ffone_rc_ptr(FFonePACore) core, int val) {
    FFONE_RETURN_ON_FAILURE(core);

    pa_threaded_mainloop_signal(ffone_pa_core_get_loop(core), val);
}

void ffone_pa_core_loop_wait(ffone_rc_ptr(FFonePACore) core) {
    FFONE_RETURN_ON_FAILURE(core);

    pa_threaded_mainloop_wait(ffone_pa_core_get_loop(core));
}

int ffone_pa_core_execute_operation(ffone_rc_ptr(FFonePACore) core, pa_operation *o) {
    FFONE_RETURN_VAL_ON_FAILURE(core && o, FFONE_ERROR_INVALID_ARG);

    pa_threaded_mainloop *loop = ffone_pa_core_get_loop(core);

    pa_operation_state_t state;
    while ((state = pa_operation_get_state(o)) == PA_OPERATION_RUNNING) {
        pa_threaded_mainloop_wait(loop);
    }

    pa_operation_unref(o);

    return (state == PA_OPERATION_DONE) ? FFONE_SUCCESS : FFONE_ERROR_CUSTOM;
}

int ffone_pa_core_load_virtual_device(
    ffone_rc_ptr(FFonePACore) core,
    const char *module,
    const char *args,
    pa_context_index_cb_t cb,
    void *userdata
) {
    FFONE_RETURN_VAL_ON_FAILURE(core && module && args && cb, FFONE_ERROR_INVALID_ARG);

    pa_threaded_mainloop_lock(core->loop);

    pa_operation *o = pa_context_load_module(
        core->context,
        module,
        args,
        cb,
        userdata
    );
    FFONE_GOTO_ON_FAILURE(o, operation_error);

    int ret = ffone_pa_core_execute_operation(core, o);

    pa_threaded_mainloop_unlock(core->loop);

    return ret;
operation_error:
    pa_threaded_mainloop_unlock(core->loop);

    return FFONE_ERROR_BAD_ALLOC;
}

int ffone_pa_core_unload_virtual_device(
    ffone_rc_ptr(FFonePACore) core,
    uint32_t idx,
    pa_context_success_cb_t cb,
    void *userdata
) {
    FFONE_RETURN_VAL_ON_FAILURE(core && cb, FFONE_ERROR_INVALID_ARG);
    FFONE_RETURN_VAL_ON_FAILURE(idx != FFONE_PA_VIRTUAL_DEVICE_INDEX_NONE, FFONE_SUCCESS);

    pa_threaded_mainloop_lock(core->loop);

    pa_operation *o = pa_context_unload_module(
        ffone_pa_core_get_context(core),
        idx,
        cb,
        userdata
    );
    FFONE_GOTO_ON_FAILURE(o, operation_error);

    int ret = ffone_pa_core_execute_operation(core, o);

    pa_threaded_mainloop_unlock(core->loop);

    return ret;
operation_error:
    pa_threaded_mainloop_unlock(core->loop);

    return FFONE_ERROR_BAD_ALLOC;
}