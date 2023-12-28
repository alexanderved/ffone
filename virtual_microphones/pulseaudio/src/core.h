#ifndef _FFONE_PA_CORE_H
#define _FFONE_PA_CORE_H

#include "rc.h"

#include <pulse/pulseaudio.h>

typedef struct FFonePACore FFonePACore;

ffone_rc(FFonePACore) ffone_pa_core_new(void);

pa_context *ffone_pa_core_get_context(ffone_rc_ptr(FFonePACore) core);

pa_threaded_mainloop *ffone_pa_core_get_loop(ffone_rc_ptr(FFonePACore) core);
void ffone_pa_core_loop_lock(ffone_rc_ptr(FFonePACore) core);
void ffone_pa_core_loop_unlock(ffone_rc_ptr(FFonePACore) core);
void ffone_pa_core_loop_signal(ffone_rc_ptr(FFonePACore) core, int val);
void ffone_pa_core_loop_wait(ffone_rc_ptr(FFonePACore) core);

int ffone_pa_core_execute_operation(ffone_rc_ptr(FFonePACore) core, pa_operation *o);

int ffone_pa_core_load_virtual_device(
    ffone_rc_ptr(FFonePACore) core,
    const char *module,
    const char *args,
    pa_context_index_cb_t cb,
    void *userdata
);
int ffone_pa_core_unload_virtual_device(
    ffone_rc_ptr(FFonePACore) core,
    uint32_t idx,
    pa_context_success_cb_t cb,
    void *userdata
);

#endif /* _FFONE_PA_CORE_H */
