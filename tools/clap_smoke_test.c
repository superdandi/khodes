// Smoke host mínimo para validar un bundle CLAP sin DAW.
// Uso: clap_smoke_test /ruta/a/Khodes.clap
//
// Verifica: símbolo clap_entry, factory, descriptor, init/activate/
// start_processing/process y silencio finito en salida. No es un host de
// producción: implementa solo lo imprescindible de la ABI CLAP.

#include <dlfcn.h>
#include <math.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <clap/clap.h>

#define FRAMES 256
#define FAIL(msg) do { fprintf(stderr, "FAIL: %s\n", (msg)); goto cleanup; } while (0)

static const void *host_get_extension(const clap_host_t *host, const char *id) {
    (void)host; (void)id;
    return NULL;
}
static void host_request_restart(const clap_host_t *host) { (void)host; }
static void host_request_process(const clap_host_t *host) { (void)host; }
static void host_request_callback(const clap_host_t *host) { (void)host; }

static const clap_host_t HOST = {
    .clap_version = CLAP_VERSION_INIT,
    .host_data = NULL,
    .name = "Khodes Smoke Host",
    .vendor = "Khodes",
    .url = "https://github.com/khodes/khodes",
    .version = "0.1",
    .get_extension = host_get_extension,
    .request_restart = host_request_restart,
    .request_process = host_request_process,
    .request_callback = host_request_callback,
};

static uint32_t in_size(const clap_input_events_t *list) { (void)list; return 0; }
static const clap_event_header_t *in_get(const clap_input_events_t *list, uint32_t i) {
    (void)list; (void)i;
    return NULL;
}
static bool out_push(const clap_output_events_t *list, const clap_event_header_t *event) {
    (void)list; (void)event;
    return true;
}

int main(int argc, char **argv) {
    void *handle = NULL;
    const clap_plugin_t *plugin = NULL;
    const clap_plugin_entry_t *entry = NULL;
    const clap_plugin_factory_t *factory = NULL;
    int result = 1;

    if (argc != 2) {
        fprintf(stderr, "Uso: %s <bundle.clap>\n", argv[0]);
        return 2;
    }

    handle = dlopen(argv[1], RTLD_NOW | RTLD_LOCAL);
    if (!handle) { fprintf(stderr, "dlopen: %s\n", dlerror()); return 1; }

    entry = (const clap_plugin_entry_t *)dlsym(handle, "clap_entry");
    if (!entry) FAIL("símbolo clap_entry no encontrado");
    printf("clap version: %u.%u.%u\n", entry->clap_version.major,
           entry->clap_version.minor, entry->clap_version.revision);

    if (!entry->init(argv[1])) FAIL("entry->init devolvió false");

    factory = (const clap_plugin_factory_t *)entry->get_factory(CLAP_PLUGIN_FACTORY_ID);
    if (!factory) FAIL("factory no disponible");

    uint32_t count = factory->get_plugin_count(factory);
    printf("plugins in factory: %u\n", count);
    if (count == 0) FAIL("factory sin plugins");

    const clap_plugin_descriptor_t *desc = factory->get_plugin_descriptor(factory, 0);
    if (!desc) FAIL("descriptor no disponible");
    printf("descriptor: id=%s name=%s vendor=%s version=%s\n",
           desc->id, desc->name, desc->vendor, desc->version);

    plugin = factory->create_plugin(factory, &HOST, desc->id);
    if (!plugin) FAIL("create_plugin devolvió NULL");
    if (!plugin->init(plugin)) FAIL("plugin->init devolvió false");
    if (!plugin->activate(plugin, 48000.0, FRAMES, FRAMES)) FAIL("activate devolvió false");
    if (!plugin->start_processing(plugin)) FAIL("start_processing devolvió false");

    float *left = calloc(FRAMES, sizeof(float));
    float *right = calloc(FRAMES, sizeof(float));
    float *l_ptr = left;
    float *r_ptr = right;
    clap_audio_buffer_t outputs[2] = {
        { .data32 = &l_ptr, .data64 = NULL, .channel_count = 1, .latency = 0, .constant_mask = 0 },
        { .data32 = &r_ptr, .data64 = NULL, .channel_count = 1, .latency = 0, .constant_mask = 0 },
    };
    clap_input_events_t in_events = { .ctx = NULL, .size = in_size, .get = in_get };
    clap_output_events_t out_events = { .ctx = NULL, .try_push = out_push };

    for (int block = 0; block < 4; ++block) {
        clap_process_t process = {
            .steady_time = block * FRAMES,
            .frames_count = FRAMES,
            .transport = NULL,
            .audio_inputs = NULL,
            .audio_outputs = outputs,
            .audio_inputs_count = 0,
            .audio_outputs_count = 2,
            .in_events = &in_events,
            .out_events = &out_events,
        };
        clap_process_status status = plugin->process(plugin, &process);
        if (status == CLAP_PROCESS_ERROR) FAIL("process devolvió CLAP_PROCESS_ERROR");
        for (int i = 0; i < FRAMES; ++i) {
            if (!isfinite(left[i]) || !isfinite(right[i])) FAIL("salida no finita");
        }
    }
    printf("process: 4 bloques de %d frames, salida finita\n", FRAMES);

    plugin->stop_processing(plugin);
    plugin->deactivate(plugin);
    plugin->destroy(plugin);
    plugin = NULL;
    entry->deinit();
    printf("OK: ciclo de vida CLAP completado\n");
    result = 0;

cleanup:
    if (plugin) {
        plugin->stop_processing(plugin);
        plugin->deactivate(plugin);
        plugin->destroy(plugin);
    }
    if (entry) entry->deinit();
    if (handle) dlclose(handle);
    return result;
}
