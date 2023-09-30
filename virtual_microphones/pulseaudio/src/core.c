#include "core.h"
#include "virtual_device.h"

#include "error.h"

#include <stdio.h>
#include <stdlib.h>

struct FFonePACore {
    pa_mainloop *loop;
    pa_context *context;
};

static void ffone_pa_core_dtor(void *opaque);

ffone_rc(FFonePACore) ffone_pa_core_new(void) {
    ffone_rc(FFonePACore) core = ffone_rc_new0(FFonePACore);
    FFONE_RETURN_VAL_ON_FAILURE(core, NULL);

    FFONE_GOTO_ON_FAILURE(core->loop = pa_mainloop_new(), error);

    pa_mainloop_api *api = pa_mainloop_get_api(core->loop);
    FFONE_GOTO_ON_FAILURE(api, error);

    FFONE_GOTO_ON_FAILURE(
        core->context = pa_context_new(api, "ffone_pa_virtual_microphone"),
        error
    );
    
    FFONE_GOTO_ON_FAILURE(
        pa_context_connect(core->context, NULL, PA_CONTEXT_NOAUTOSPAWN, NULL) >= 0,
        error
    );

    pa_context_state_t state = PA_CONTEXT_UNCONNECTED;
    while (state != PA_CONTEXT_READY) {
        pa_mainloop_iterate(core->loop, 1, NULL);

        state = pa_context_get_state(core->context);
        if (state == PA_CONTEXT_FAILED || state == PA_CONTEXT_TERMINATED) goto error;
    }

    ffone_rc_set_dtor(core, ffone_pa_core_dtor);

    return core;
error:
    fprintf(stderr, "Failed to create FFonePACore\n");

    if (core->context) {
        if (pa_context_get_state(core->context) == PA_CONTEXT_READY) {
            pa_context_disconnect(core->context);
        }

        pa_context_unref(core->context);
    }

    if (core->loop) pa_mainloop_free(core->loop);
    if (core) ffone_rc_unref(core);

    return NULL;
}

static void ffone_pa_core_dtor(void *opaque) {
    FFonePACore *core = opaque;
    FFONE_RETURN_ON_FAILURE(core);

    if (core->context) {
        if (pa_context_get_state(core->context) == PA_CONTEXT_READY) {
            pa_context_disconnect(core->context);
        }
        pa_context_unref(core->context);
        core->context = NULL;
    }

    if (core->loop) {
        pa_mainloop_quit(core->loop, 0);
        pa_mainloop_free(core->loop);
        core->loop = NULL;
    }
}

pa_context *ffone_pa_core_get_context(ffone_rc_ptr(FFonePACore) core) {
    FFONE_RETURN_VAL_ON_FAILURE(core, NULL);
    FFONE_RETURN_VAL_ON_FAILURE(!ffone_rc_is_destructed(core), NULL);

    return core->context;
}

pa_mainloop *ffone_pa_core_get_loop(ffone_rc_ptr(FFonePACore)core) {
    FFONE_RETURN_VAL_ON_FAILURE(core, NULL);
    FFONE_RETURN_VAL_ON_FAILURE(!ffone_rc_is_destructed(core), NULL);

    return core->loop;
}

int ffone_pa_core_execute_operation(ffone_rc_ptr(FFonePACore) core, pa_operation *o) {
    FFONE_RETURN_VAL_ON_FAILURE(core && o, FFONE_ERROR_INVALID_ARG);
    FFONE_RETURN_VAL_ON_FAILURE(!ffone_rc_is_destructed(core), FFONE_ERROR_BAD_STATE);

    pa_operation_state_t state;
    while ((state = pa_operation_get_state(o)) == PA_OPERATION_RUNNING) {
        if (ffone_pa_core_iterate(core, 1) < 0) {
            pa_operation_cancel(o);
            continue;
        }
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
    FFONE_RETURN_VAL_ON_FAILURE(
        !ffone_rc_is_destructed(core) && core->context,
        FFONE_ERROR_BAD_STATE
    );

    pa_operation *o = pa_context_load_module(
        core->context,
        module,
        args,
        cb,
        userdata
    );
    FFONE_RETURN_VAL_ON_FAILURE(o, FFONE_ERROR_BAD_ALLOC);

    return ffone_pa_core_execute_operation(core, o);
}

int ffone_pa_core_unload_virtual_device(
    ffone_rc_ptr(FFonePACore) core,
    uint32_t idx,
    pa_context_success_cb_t cb,
    void *userdata
) {
    FFONE_RETURN_VAL_ON_FAILURE(core && cb, FFONE_ERROR_INVALID_ARG);
    FFONE_RETURN_VAL_ON_FAILURE(
        !ffone_rc_is_destructed(core) && core->context,
        FFONE_ERROR_BAD_STATE
    );
    FFONE_RETURN_VAL_ON_FAILURE(idx != FFONE_PA_VIRTUAL_DEVICE_INDEX_NONE, FFONE_SUCCESS);

    pa_operation *o = pa_context_unload_module(
        core->context,
        idx,
        cb,
        userdata
    );
    FFONE_RETURN_VAL_ON_FAILURE(o, FFONE_ERROR_BAD_ALLOC);

    return ffone_pa_core_execute_operation(core, o);
}

int ffone_pa_core_iterate(ffone_rc_ptr(FFonePACore) core, int block) {
    FFONE_RETURN_VAL_ON_FAILURE(core, FFONE_ERROR_INVALID_ARG);
    FFONE_RETURN_VAL_ON_FAILURE(
        !ffone_rc_is_destructed(core) && core->loop,
        FFONE_ERROR_BAD_STATE
    );

    return FFONE_ERROR(pa_mainloop_iterate(core->loop, block, NULL));
}