use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::time::Instant;

use shellow_core::{
    AuthenticationKind, HostProfile, ShellowEngine, renderer::RendererOverlayState,
};

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_create() -> *mut ShellowEngine {
    match catch_unwind(AssertUnwindSafe(|| {
        Box::into_raw(Box::new(ShellowEngine::new()))
    })) {
        Ok(engine) => engine,
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_destroy(engine: *mut ShellowEngine) {
    if !engine.is_null() {
        unsafe {
            drop(Box::from_raw(engine));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_snapshot_json(engine: *const ShellowEngine) -> *mut c_char {
    with_engine(engine, |engine| encode_json(&engine.snapshot()))
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_render_frame_json(
    engine: *const ShellowEngine,
    width_px: u32,
    height_px: u32,
) -> *mut c_char {
    with_engine(engine, |engine| {
        encode_json(&engine.render_frame(width_px, height_px))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_render_frame_viewport_json(
    engine: *const ShellowEngine,
    width_px: u32,
    height_px: u32,
    first_row: u32,
    row_count: u32,
) -> *mut c_char {
    with_engine(engine, |engine| {
        encode_json(&engine.render_frame_viewport(width_px, height_px, first_row, row_count))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_render_surface_frame_presented(
    engine: *const ShellowEngine,
    width_px: u32,
    height_px: u32,
    first_row: u32,
    row_count: u32,
) -> bool {
    if engine.is_null() {
        return false;
    }

    catch_unwind(AssertUnwindSafe(|| unsafe {
        (*engine).render_surface_frame_presented(width_px, height_px, first_row, row_count)
    }))
    .unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_renderer_info_json(engine: *const ShellowEngine) -> *mut c_char {
    with_engine(engine, |engine| encode_json(&engine.renderer_info()))
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_live_shell_event_revision(engine: *const ShellowEngine) -> u64 {
    if engine.is_null() {
        return 0;
    }

    catch_unwind(AssertUnwindSafe(|| unsafe {
        (*engine).live_shell_event_revision()
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_set_renderer_overlay_json(
    engine: *mut ShellowEngine,
    overlay_json: *const c_char,
) -> *mut c_char {
    with_engine_mut(engine, |engine| {
        let overlay_json = read_c_string(overlay_json);
        match serde_json::from_str::<RendererOverlayState>(&overlay_json) {
            Ok(state) => encode_json(&engine.set_renderer_overlay(state)),
            Err(error) => error_json(&format!("renderer overlay json decode failed: {error}")),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_attach_core_animation_layer_json(
    engine: *mut ShellowEngine,
    raw_handle: u64,
    width_px: u32,
    height_px: u32,
) -> *mut c_char {
    with_engine_mut(engine, |engine| {
        encode_json(
            &engine.attach_core_animation_layer_renderer_surface(raw_handle, width_px, height_px),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_attach_android_native_window_json(
    engine: *mut ShellowEngine,
    raw_handle: u64,
    width_px: u32,
    height_px: u32,
) -> *mut c_char {
    with_engine_mut(engine, |engine| {
        encode_json(
            &engine.attach_android_native_window_renderer_surface(raw_handle, width_px, height_px),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_detach_renderer_surface_json(
    engine: *mut ShellowEngine,
) -> *mut c_char {
    with_engine_mut(engine, |engine| {
        encode_json(&engine.detach_renderer_surface())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_send_command_json(
    engine: *mut ShellowEngine,
    input: *const c_char,
) -> *mut c_char {
    with_engine_mut(engine, |engine| {
        let input = read_c_string(input);
        encode_json(&engine.send_command(&input))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_send_terminal_input_json(
    engine: *mut ShellowEngine,
    input: *const c_char,
) -> *mut c_char {
    with_engine_mut(engine, |engine| {
        let input = read_c_string(input);
        encode_json(&engine.send_terminal_input(&input))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_resize_terminal_json(
    engine: *mut ShellowEngine,
    cols: u32,
    rows: u32,
) -> *mut c_char {
    with_engine_mut(engine, |engine| {
        encode_json(&engine.resize_terminal(cols, rows))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_clear_terminal_json(engine: *mut ShellowEngine) -> *mut c_char {
    with_engine_mut(engine, |engine| encode_json(&engine.clear_terminal()))
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_reset_terminal_json(engine: *mut ShellowEngine) -> *mut c_char {
    with_engine_mut(engine, |engine| encode_json(&engine.reset_terminal()))
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_connect_preview_json(
    engine: *mut ShellowEngine,
    name: *const c_char,
    host: *const c_char,
    port: u16,
    username: *const c_char,
    trusted_host_key_sha256: *const c_char,
    auth_kind: u8,
) -> *mut c_char {
    with_engine_mut(engine, |engine| {
        let profile = HostProfile {
            name: read_c_string(name),
            host: read_c_string(host),
            port,
            username: read_c_string(username),
            authentication: match auth_kind {
                1 => AuthenticationKind::PrivateKey,
                _ => AuthenticationKind::Password,
            },
            trusted_host_key_sha256: read_optional_c_string(trusted_host_key_sha256),
        };
        encode_json(&engine.connect_preview(profile))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_connect_password_exec_json(
    engine: *mut ShellowEngine,
    name: *const c_char,
    host: *const c_char,
    port: u16,
    username: *const c_char,
    trusted_host_key_sha256: *const c_char,
    password: *const c_char,
    command: *const c_char,
) -> *mut c_char {
    with_engine_mut(engine, |engine| {
        let profile = HostProfile {
            name: read_c_string(name),
            host: read_c_string(host),
            port,
            username: read_c_string(username),
            authentication: AuthenticationKind::Password,
            trusted_host_key_sha256: read_optional_c_string(trusted_host_key_sha256),
        };
        encode_json(&engine.connect_password_exec(
            profile,
            read_c_string(password),
            read_c_string(command),
        ))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_connect_private_key_exec_json(
    engine: *mut ShellowEngine,
    name: *const c_char,
    host: *const c_char,
    port: u16,
    username: *const c_char,
    trusted_host_key_sha256: *const c_char,
    private_key_pem: *const c_char,
    passphrase: *const c_char,
    command: *const c_char,
) -> *mut c_char {
    with_engine_mut(engine, |engine| {
        let profile = HostProfile {
            name: read_c_string(name),
            host: read_c_string(host),
            port,
            username: read_c_string(username),
            authentication: AuthenticationKind::PrivateKey,
            trusted_host_key_sha256: read_optional_c_string(trusted_host_key_sha256),
        };
        encode_json(&engine.connect_private_key_exec(
            profile,
            read_c_string(private_key_pem),
            read_optional_c_string(passphrase),
            read_c_string(command),
        ))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_start_password_shell_json(
    engine: *mut ShellowEngine,
    name: *const c_char,
    host: *const c_char,
    port: u16,
    username: *const c_char,
    trusted_host_key_sha256: *const c_char,
    password: *const c_char,
) -> *mut c_char {
    with_engine_mut(engine, |engine| {
        let profile = HostProfile {
            name: read_c_string(name),
            host: read_c_string(host),
            port,
            username: read_c_string(username),
            authentication: AuthenticationKind::Password,
            trusted_host_key_sha256: read_optional_c_string(trusted_host_key_sha256),
        };
        encode_json(&engine.start_password_shell(profile, read_c_string(password)))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_start_private_key_shell_json(
    engine: *mut ShellowEngine,
    name: *const c_char,
    host: *const c_char,
    port: u16,
    username: *const c_char,
    trusted_host_key_sha256: *const c_char,
    private_key_pem: *const c_char,
    passphrase: *const c_char,
) -> *mut c_char {
    with_engine_mut(engine, |engine| {
        let profile = HostProfile {
            name: read_c_string(name),
            host: read_c_string(host),
            port,
            username: read_c_string(username),
            authentication: AuthenticationKind::PrivateKey,
            trusted_host_key_sha256: read_optional_c_string(trusted_host_key_sha256),
        };
        encode_json(&engine.start_private_key_shell(
            profile,
            read_c_string(private_key_pem),
            read_optional_c_string(passphrase),
        ))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_poll_live_shell_json(engine: *mut ShellowEngine) -> *mut c_char {
    with_engine_mut(engine, |engine| encode_json(&engine.poll_live_shell()))
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_disconnect_live_shell_json(
    engine: *mut ShellowEngine,
) -> *mut c_char {
    with_engine_mut(engine, |engine| {
        encode_json(&engine.disconnect_live_shell())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_codex_snapshot_json(engine: *const ShellowEngine) -> *mut c_char {
    let started = Instant::now();
    with_engine(engine, |engine| {
        encode_codex_json("snapshot", started, &engine.codex_snapshot())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_codex_event_revision(engine: *const ShellowEngine) -> u64 {
    if engine.is_null() {
        return 0;
    }

    catch_unwind(AssertUnwindSafe(|| unsafe {
        (*engine).codex_event_revision()
    }))
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_start_codex_password_json(
    engine: *mut ShellowEngine,
    name: *const c_char,
    host: *const c_char,
    port: u16,
    username: *const c_char,
    trusted_host_key_sha256: *const c_char,
    password: *const c_char,
    cwd: *const c_char,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        let profile = HostProfile {
            name: read_c_string(name),
            host: read_c_string(host),
            port,
            username: read_c_string(username),
            authentication: AuthenticationKind::Password,
            trusted_host_key_sha256: read_optional_c_string(trusted_host_key_sha256),
        };
        encode_codex_json(
            "start_password",
            started,
            &engine.start_codex_password(
                profile,
                read_c_string(password),
                read_optional_c_string(cwd),
            ),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_start_codex_private_key_json(
    engine: *mut ShellowEngine,
    name: *const c_char,
    host: *const c_char,
    port: u16,
    username: *const c_char,
    trusted_host_key_sha256: *const c_char,
    private_key_pem: *const c_char,
    passphrase: *const c_char,
    cwd: *const c_char,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        let profile = HostProfile {
            name: read_c_string(name),
            host: read_c_string(host),
            port,
            username: read_c_string(username),
            authentication: AuthenticationKind::PrivateKey,
            trusted_host_key_sha256: read_optional_c_string(trusted_host_key_sha256),
        };
        encode_codex_json(
            "start_private_key",
            started,
            &engine.start_codex_private_key(
                profile,
                read_c_string(private_key_pem),
                read_optional_c_string(passphrase),
                read_optional_c_string(cwd),
            ),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_poll_codex_json(engine: *mut ShellowEngine) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json("poll", started, &engine.poll_codex())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_send_codex_message_json(
    engine: *mut ShellowEngine,
    message: *const c_char,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json(
            "send_message",
            started,
            &engine.send_codex_message(&read_c_string(message)),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_update_codex_settings_json(
    engine: *mut ShellowEngine,
    model: *const c_char,
    approval_policy: *const c_char,
    sandbox: *const c_char,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json(
            "update_settings",
            started,
            &engine.update_codex_settings(
                read_optional_c_string(model).as_deref(),
                read_optional_c_string(approval_policy).as_deref(),
                read_optional_c_string(sandbox).as_deref(),
            ),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_browse_codex_directory_json(
    engine: *mut ShellowEngine,
    path: *const c_char,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json(
            "browse_directory",
            started,
            &engine.browse_codex_directory(&read_c_string(path)),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_list_codex_threads_json(
    engine: *mut ShellowEngine,
    cwd: *const c_char,
    search_term: *const c_char,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json(
            "list_threads",
            started,
            &engine.list_codex_threads(
                read_optional_c_string(cwd).as_deref(),
                read_optional_c_string(search_term).as_deref(),
            ),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_list_codex_threads_page_json(
    engine: *mut ShellowEngine,
    cwd: *const c_char,
    search_term: *const c_char,
    cursor: *const c_char,
    archived: bool,
    append: bool,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json(
            "list_threads_page",
            started,
            &engine.list_codex_threads_page(
                read_optional_c_string(cwd).as_deref(),
                read_optional_c_string(search_term).as_deref(),
                read_optional_c_string(cursor).as_deref(),
                archived,
                append,
            ),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_start_codex_thread_json(
    engine: *mut ShellowEngine,
    cwd: *const c_char,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json(
            "start_thread",
            started,
            &engine.start_codex_thread(read_optional_c_string(cwd).as_deref()),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_resume_codex_thread_json(
    engine: *mut ShellowEngine,
    thread_id: *const c_char,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json(
            "resume_thread",
            started,
            &engine.resume_codex_thread(&read_c_string(thread_id)),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_read_codex_thread_json(
    engine: *mut ShellowEngine,
    thread_id: *const c_char,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json(
            "read_thread",
            started,
            &engine.read_codex_thread(&read_c_string(thread_id)),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_load_more_codex_thread_turns_json(
    engine: *mut ShellowEngine,
    thread_id: *const c_char,
    cursor: *const c_char,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json(
            "load_more_turns",
            started,
            &engine.load_more_codex_thread_turns(
                &read_c_string(thread_id),
                read_optional_c_string(cursor).as_deref(),
            ),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_rename_codex_thread_json(
    engine: *mut ShellowEngine,
    thread_id: *const c_char,
    name: *const c_char,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json(
            "rename_thread",
            started,
            &engine.rename_codex_thread(&read_c_string(thread_id), &read_c_string(name)),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_archive_codex_thread_json(
    engine: *mut ShellowEngine,
    thread_id: *const c_char,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json(
            "archive_thread",
            started,
            &engine.archive_codex_thread(&read_c_string(thread_id)),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_unarchive_codex_thread_json(
    engine: *mut ShellowEngine,
    thread_id: *const c_char,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json(
            "unarchive_thread",
            started,
            &engine.unarchive_codex_thread(&read_c_string(thread_id)),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_delete_codex_thread_json(
    engine: *mut ShellowEngine,
    thread_id: *const c_char,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json(
            "delete_thread",
            started,
            &engine.delete_codex_thread(&read_c_string(thread_id)),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_fork_codex_thread_json(
    engine: *mut ShellowEngine,
    thread_id: *const c_char,
    cwd: *const c_char,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json(
            "fork_thread",
            started,
            &engine.fork_codex_thread(
                &read_c_string(thread_id),
                read_optional_c_string(cwd).as_deref(),
            ),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_interrupt_codex_turn_json(
    engine: *mut ShellowEngine,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json("interrupt_turn", started, &engine.interrupt_codex_turn())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_answer_codex_approval_json(
    engine: *mut ShellowEngine,
    request_id: *const c_char,
    decision: *const c_char,
) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json(
            "answer_approval",
            started,
            &engine.answer_codex_approval(&read_c_string(request_id), &read_c_string(decision)),
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_engine_disconnect_codex_json(engine: *mut ShellowEngine) -> *mut c_char {
    let started = Instant::now();
    with_engine_mut(engine, |engine| {
        encode_codex_json("disconnect", started, &engine.disconnect_codex())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn shellow_string_free(value: *mut c_char) {
    if !value.is_null() {
        unsafe {
            drop(CString::from_raw(value));
        }
    }
}

fn with_engine<F>(engine: *const ShellowEngine, body: F) -> *mut c_char
where
    F: FnOnce(&ShellowEngine) -> *mut c_char,
{
    if engine.is_null() {
        return error_json("engine pointer was null");
    }

    match catch_unwind(AssertUnwindSafe(|| body(unsafe { &*engine }))) {
        Ok(value) => value,
        Err(_) => error_json("shellow ffi panic"),
    }
}

fn with_engine_mut<F>(engine: *mut ShellowEngine, body: F) -> *mut c_char
where
    F: FnOnce(&mut ShellowEngine) -> *mut c_char,
{
    if engine.is_null() {
        return error_json("engine pointer was null");
    }

    match catch_unwind(AssertUnwindSafe(|| body(unsafe { &mut *engine }))) {
        Ok(value) => value,
        Err(_) => error_json("shellow ffi panic"),
    }
}

fn read_c_string(value: *const c_char) -> String {
    if value.is_null() {
        return String::new();
    }

    unsafe { CStr::from_ptr(value) }
        .to_string_lossy()
        .into_owned()
}

fn read_optional_c_string(value: *const c_char) -> Option<String> {
    let value = read_c_string(value).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

fn encode_json<T: serde::Serialize>(value: &T) -> *mut c_char {
    match serde_json::to_string(value) {
        Ok(json) => into_c_string(json),
        Err(error) => error_json(&format!("json encode failed: {error}")),
    }
}

fn encode_codex_json<T: serde::Serialize>(label: &str, started: Instant, value: &T) -> *mut c_char {
    let encode_started = Instant::now();
    match serde_json::to_string(value) {
        Ok(json) => {
            println!(
                "[Shellow Codex] ffi {label} bytes={} encode_ms={} total_ms={}",
                json.len(),
                encode_started.elapsed().as_millis(),
                started.elapsed().as_millis()
            );
            into_c_string(json)
        }
        Err(error) => error_json(&format!("json encode failed: {error}")),
    }
}

fn error_json(message: &str) -> *mut c_char {
    let escaped = serde_json::to_string(message).unwrap_or_else(|_| "\"unknown\"".to_string());
    into_c_string(format!("{{\"error\":{escaped}}}"))
}

fn into_c_string(value: String) -> *mut c_char {
    match CString::new(value) {
        Ok(value) => value.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}
