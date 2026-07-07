use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;

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
pub extern "C" fn shellow_engine_renderer_info_json(engine: *const ShellowEngine) -> *mut c_char {
    with_engine(engine, |engine| encode_json(&engine.renderer_info()))
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
