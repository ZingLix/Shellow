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
char *shellow_engine_set_terminal_theme_json(ShellowEngine *engine, const char *theme_id);
void shellow_engine_set_transport_options(ShellowEngine *engine, uint64_t keepalive_interval_secs, bool remote_port_detection_enabled);
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
char *shellow_engine_connect_private_key_exec_json(
    ShellowEngine *engine,
    const char *name,
    const char *host,
    uint16_t port,
    const char *username,
    const char *trusted_host_key_sha256,
    const char *private_key_pem,
    const char *passphrase,
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
char *shellow_engine_dismiss_detected_remote_port_json(ShellowEngine *engine, uint16_t port);
char *shellow_engine_disconnect_live_shell_json(ShellowEngine *engine);
char *shellow_engine_codex_snapshot_json(const ShellowEngine *engine);
uint64_t shellow_engine_codex_event_revision(const ShellowEngine *engine);
char *shellow_engine_start_codex_password_json(
    ShellowEngine *engine,
    const char *name,
    const char *host,
    uint16_t port,
    const char *username,
    const char *trusted_host_key_sha256,
    const char *password,
    const char *cwd
);
char *shellow_engine_start_codex_private_key_json(
    ShellowEngine *engine,
    const char *name,
    const char *host,
    uint16_t port,
    const char *username,
    const char *trusted_host_key_sha256,
    const char *private_key_pem,
    const char *passphrase,
    const char *cwd
);
char *shellow_engine_poll_codex_json(ShellowEngine *engine);
char *shellow_engine_send_codex_message_json(ShellowEngine *engine, const char *message);
char *shellow_engine_update_codex_settings_json(ShellowEngine *engine, const char *model, const char *reasoning_effort, const char *service_tier, const char *approval_policy, const char *sandbox);
char *shellow_engine_browse_codex_directory_json(ShellowEngine *engine, const char *path);
char *shellow_engine_list_codex_threads_json(ShellowEngine *engine, const char *cwd, const char *search_term);
char *shellow_engine_list_codex_threads_page_json(ShellowEngine *engine, const char *cwd, const char *search_term, const char *cursor, bool archived, bool append);
char *shellow_engine_start_codex_thread_json(ShellowEngine *engine, const char *cwd);
char *shellow_engine_resume_codex_thread_json(ShellowEngine *engine, const char *thread_id);
char *shellow_engine_read_codex_thread_json(ShellowEngine *engine, const char *thread_id);
char *shellow_engine_load_more_codex_thread_turns_json(ShellowEngine *engine, const char *thread_id, const char *cursor);
char *shellow_engine_rename_codex_thread_json(ShellowEngine *engine, const char *thread_id, const char *name);
char *shellow_engine_archive_codex_thread_json(ShellowEngine *engine, const char *thread_id);
char *shellow_engine_unarchive_codex_thread_json(ShellowEngine *engine, const char *thread_id);
char *shellow_engine_delete_codex_thread_json(ShellowEngine *engine, const char *thread_id);
char *shellow_engine_fork_codex_thread_json(ShellowEngine *engine, const char *thread_id, const char *cwd);
char *shellow_engine_interrupt_codex_turn_json(ShellowEngine *engine);
char *shellow_engine_answer_codex_approval_json(ShellowEngine *engine, const char *request_id, const char *decision);
char *shellow_engine_disconnect_codex_json(ShellowEngine *engine);
char *shellow_engine_claude_snapshot_json(const ShellowEngine *engine);
uint64_t shellow_engine_claude_event_revision(const ShellowEngine *engine);
char *shellow_engine_start_claude_password_json(
    ShellowEngine *engine,
    const char *name,
    const char *host,
    uint16_t port,
    const char *username,
    const char *trusted_host_key_sha256,
    const char *password,
    const char *cwd,
    const char *session_id
);
char *shellow_engine_start_claude_private_key_json(
    ShellowEngine *engine,
    const char *name,
    const char *host,
    uint16_t port,
    const char *username,
    const char *trusted_host_key_sha256,
    const char *private_key_pem,
    const char *passphrase,
    const char *cwd,
    const char *session_id
);
char *shellow_engine_poll_claude_json(ShellowEngine *engine);
char *shellow_engine_send_claude_message_json(ShellowEngine *engine, const char *message);
char *shellow_engine_update_claude_settings_json(ShellowEngine *engine, const char *model, const char *permission_mode);
char *shellow_engine_interrupt_claude_turn_json(ShellowEngine *engine);
char *shellow_engine_answer_claude_approval_json(ShellowEngine *engine, const char *request_id, const char *decision);
char *shellow_engine_disconnect_claude_json(ShellowEngine *engine);

void shellow_string_free(char *value);

#ifdef __cplusplus
}
#endif

#endif
