#ifndef SHELLOW_CORE_H
#define SHELLOW_CORE_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef void ShellowEngine;

ShellowEngine *shellow_engine_create(void);
void shellow_engine_destroy(ShellowEngine *engine);

char *shellow_engine_snapshot_json(const ShellowEngine *engine);
char *shellow_engine_render_frame_json(const ShellowEngine *engine, uint32_t width_px, uint32_t height_px);
char *shellow_engine_render_frame_viewport_json(const ShellowEngine *engine, uint32_t width_px, uint32_t height_px, uint32_t first_row, uint32_t row_count);
bool shellow_engine_render_surface_frame_presented(const ShellowEngine *engine, uint32_t width_px, uint32_t height_px, uint32_t first_row, uint32_t row_count);
char *shellow_engine_renderer_info_json(const ShellowEngine *engine);
uint64_t shellow_engine_live_shell_event_revision(const ShellowEngine *engine);
char *shellow_engine_set_renderer_overlay_json(ShellowEngine *engine, const char *overlay_json);
char *shellow_engine_attach_core_animation_layer_json(ShellowEngine *engine, uint64_t raw_handle, uint32_t width_px, uint32_t height_px);
char *shellow_engine_attach_android_native_window_json(ShellowEngine *engine, uint64_t raw_handle, uint32_t width_px, uint32_t height_px);
char *shellow_engine_detach_renderer_surface_json(ShellowEngine *engine);
char *shellow_engine_send_command_json(ShellowEngine *engine, const char *input);
char *shellow_engine_send_terminal_input_json(ShellowEngine *engine, const char *input);
char *shellow_engine_resize_terminal_json(ShellowEngine *engine, uint32_t cols, uint32_t rows);
char *shellow_engine_clear_terminal_json(ShellowEngine *engine);
char *shellow_engine_reset_terminal_json(ShellowEngine *engine);
char *shellow_engine_connect_preview_json(
    ShellowEngine *engine,
    const char *name,
    const char *host,
    uint16_t port,
    const char *username,
    const char *trusted_host_key_sha256,
    uint8_t auth_kind
);
char *shellow_engine_connect_password_exec_json(
    ShellowEngine *engine,
    const char *name,
    const char *host,
    uint16_t port,
    const char *username,
    const char *trusted_host_key_sha256,
    const char *password,
    const char *command
);
char *shellow_engine_start_password_shell_json(
    ShellowEngine *engine,
    const char *name,
    const char *host,
    uint16_t port,
    const char *username,
    const char *trusted_host_key_sha256,
    const char *password
);
char *shellow_engine_start_private_key_shell_json(
    ShellowEngine *engine,
    const char *name,
    const char *host,
    uint16_t port,
    const char *username,
    const char *trusted_host_key_sha256,
    const char *private_key_pem,
    const char *passphrase
);
char *shellow_engine_poll_live_shell_json(ShellowEngine *engine);
char *shellow_engine_disconnect_live_shell_json(ShellowEngine *engine);

void shellow_string_free(char *value);

#ifdef __cplusplus
}
#endif

#endif
