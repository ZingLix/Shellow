#include <jni.h>
#include <stdint.h>

#include <android/native_window_jni.h>
#include <cstring>
#include <mutex>
#include <string>
#include <unordered_map>
#include <vector>

#include "ShellowCore.h"

namespace {

std::mutex g_surface_windows_mutex;
std::unordered_map<jlong, ANativeWindow *> g_surface_windows;

ShellowEngine *engineFromHandle(jlong handle) {
    return reinterpret_cast<ShellowEngine *>(static_cast<intptr_t>(handle));
}

void appendUtf8(std::string &output, uint32_t codepoint) {
    if (codepoint > 0x10FFFF || (codepoint >= 0xD800 && codepoint <= 0xDFFF)) {
        codepoint = 0xFFFD;
    }

    if (codepoint <= 0x7F) {
        output.push_back(static_cast<char>(codepoint));
    } else if (codepoint <= 0x7FF) {
        output.push_back(static_cast<char>(0xC0 | (codepoint >> 6)));
        output.push_back(static_cast<char>(0x80 | (codepoint & 0x3F)));
    } else if (codepoint <= 0xFFFF) {
        output.push_back(static_cast<char>(0xE0 | (codepoint >> 12)));
        output.push_back(static_cast<char>(0x80 | ((codepoint >> 6) & 0x3F)));
        output.push_back(static_cast<char>(0x80 | (codepoint & 0x3F)));
    } else {
        output.push_back(static_cast<char>(0xF0 | (codepoint >> 18)));
        output.push_back(static_cast<char>(0x80 | ((codepoint >> 12) & 0x3F)));
        output.push_back(static_cast<char>(0x80 | ((codepoint >> 6) & 0x3F)));
        output.push_back(static_cast<char>(0x80 | (codepoint & 0x3F)));
    }
}

void appendUtf16(std::vector<jchar> &output, uint32_t codepoint) {
    if (codepoint > 0x10FFFF || (codepoint >= 0xD800 && codepoint <= 0xDFFF)) {
        codepoint = 0xFFFD;
    }

    if (codepoint <= 0xFFFF) {
        output.push_back(static_cast<jchar>(codepoint));
    } else {
        codepoint -= 0x10000;
        output.push_back(static_cast<jchar>(0xD800 | (codepoint >> 10)));
        output.push_back(static_cast<jchar>(0xDC00 | (codepoint & 0x3FF)));
    }
}

std::vector<jchar> utf8ToUtf16(const char *value) {
    std::vector<jchar> result;
    if (value == nullptr) {
        return result;
    }

    const auto *bytes = reinterpret_cast<const unsigned char *>(value);
    const size_t length = std::strlen(value);
    size_t index = 0;
    while (index < length) {
        uint32_t codepoint = 0xFFFD;
        const unsigned char first = bytes[index++];

        if (first < 0x80) {
            codepoint = first;
        } else if (
            (first & 0xE0) == 0xC0 &&
            index < length &&
            (bytes[index] & 0xC0) == 0x80
        ) {
            codepoint = ((first & 0x1F) << 6) | (bytes[index] & 0x3F);
            index += 1;
            if (codepoint < 0x80) {
                codepoint = 0xFFFD;
            }
        } else if (
            (first & 0xF0) == 0xE0 &&
            index + 1 < length &&
            (bytes[index] & 0xC0) == 0x80 &&
            (bytes[index + 1] & 0xC0) == 0x80
        ) {
            codepoint =
                ((first & 0x0F) << 12) |
                ((bytes[index] & 0x3F) << 6) |
                (bytes[index + 1] & 0x3F);
            index += 2;
            if (codepoint < 0x800 || (codepoint >= 0xD800 && codepoint <= 0xDFFF)) {
                codepoint = 0xFFFD;
            }
        } else if (
            (first & 0xF8) == 0xF0 &&
            index + 2 < length &&
            (bytes[index] & 0xC0) == 0x80 &&
            (bytes[index + 1] & 0xC0) == 0x80 &&
            (bytes[index + 2] & 0xC0) == 0x80
        ) {
            codepoint =
                ((first & 0x07) << 18) |
                ((bytes[index] & 0x3F) << 12) |
                ((bytes[index + 1] & 0x3F) << 6) |
                (bytes[index + 2] & 0x3F);
            index += 3;
            if (codepoint < 0x10000 || codepoint > 0x10FFFF) {
                codepoint = 0xFFFD;
            }
        }

        appendUtf16(result, codepoint);
    }
    return result;
}

jstring newStringFromUtf8(JNIEnv *env, const char *value) {
    std::vector<jchar> utf16 = utf8ToUtf16(value);
    static const jchar empty[] = {0};
    return env->NewString(
        utf16.empty() ? empty : utf16.data(),
        static_cast<jsize>(utf16.size())
    );
}

std::string readString(JNIEnv *env, jstring value) {
    if (value == nullptr) {
        return "";
    }

    const jchar *chars = env->GetStringChars(value, nullptr);
    if (chars == nullptr) {
        return "";
    }

    std::string result;
    const jsize length = env->GetStringLength(value);
    result.reserve(static_cast<size_t>(length));
    for (jsize index = 0; index < length; index++) {
        uint32_t codepoint = chars[index];
        if (codepoint >= 0xD800 && codepoint <= 0xDBFF && index + 1 < length) {
            const uint32_t low = chars[index + 1];
            if (low >= 0xDC00 && low <= 0xDFFF) {
                codepoint = 0x10000 + (((codepoint - 0xD800) << 10) | (low - 0xDC00));
                index += 1;
            } else {
                codepoint = 0xFFFD;
            }
        } else if (codepoint >= 0xDC00 && codepoint <= 0xDFFF) {
            codepoint = 0xFFFD;
        }
        appendUtf8(result, codepoint);
    }
    env->ReleaseStringChars(value, chars);
    return result;
}

jstring takeJson(JNIEnv *env, char *value) {
    if (value == nullptr) {
        return newStringFromUtf8(env, "{\"error\":\"native returned null\"}");
    }

    jstring result = newStringFromUtf8(env, value);
    shellow_string_free(value);
    return result;
}

void releaseSurfaceWindowLocked(jlong handle) {
    auto window = g_surface_windows.find(handle);
    if (window == g_surface_windows.end()) {
        return;
    }

    ANativeWindow_release(window->second);
    g_surface_windows.erase(window);
}

void replaceSurfaceWindow(jlong handle, ANativeWindow *window) {
    std::lock_guard<std::mutex> lock(g_surface_windows_mutex);
    releaseSurfaceWindowLocked(handle);
    g_surface_windows[handle] = window;
}

void releaseSurfaceWindow(jlong handle) {
    std::lock_guard<std::mutex> lock(g_surface_windows_mutex);
    releaseSurfaceWindowLocked(handle);
}

}  // namespace

extern "C" JNIEXPORT jlong JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeCreate(JNIEnv *, jobject) {
    return static_cast<jlong>(reinterpret_cast<intptr_t>(shellow_engine_create()));
}

extern "C" JNIEXPORT void JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeDestroy(JNIEnv *, jobject, jlong handle) {
    ShellowEngine *engine = engineFromHandle(handle);
    if (engine != nullptr) {
        char *detached = shellow_engine_detach_renderer_surface_json(engine);
        if (detached != nullptr) {
            shellow_string_free(detached);
        }
    }
    releaseSurfaceWindow(handle);
    shellow_engine_destroy(engine);
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeSnapshotJson(JNIEnv *env, jobject, jlong handle) {
    return takeJson(env, shellow_engine_snapshot_json(engineFromHandle(handle)));
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeRenderFrameJson(
    JNIEnv *env,
    jobject,
    jlong handle,
    jint widthPx,
    jint heightPx
) {
    return takeJson(
        env,
        shellow_engine_render_frame_json(
            engineFromHandle(handle),
            static_cast<uint32_t>(widthPx),
            static_cast<uint32_t>(heightPx)
        )
    );
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeRenderFrameViewportJson(
    JNIEnv *env,
    jobject,
    jlong handle,
    jint widthPx,
    jint heightPx,
    jint firstRow,
    jint rowCount
) {
    return takeJson(
        env,
        shellow_engine_render_frame_viewport_json(
            engineFromHandle(handle),
            static_cast<uint32_t>(widthPx),
            static_cast<uint32_t>(heightPx),
            static_cast<uint32_t>(firstRow),
            static_cast<uint32_t>(rowCount)
        )
    );
}

extern "C" JNIEXPORT jboolean JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeRenderSurfaceFramePresented(
    JNIEnv *,
    jobject,
    jlong handle,
    jint widthPx,
    jint heightPx,
    jint firstRow,
    jint rowCount
) {
    return shellow_engine_render_surface_frame_presented(
        engineFromHandle(handle),
        static_cast<uint32_t>(widthPx),
        static_cast<uint32_t>(heightPx),
        static_cast<uint32_t>(firstRow),
        static_cast<uint32_t>(rowCount)
    ) ? JNI_TRUE : JNI_FALSE;
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeRendererInfoJson(JNIEnv *env, jobject, jlong handle) {
    return takeJson(env, shellow_engine_renderer_info_json(engineFromHandle(handle)));
}

extern "C" JNIEXPORT jlong JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeLiveShellEventRevision(JNIEnv *, jobject, jlong handle) {
    return static_cast<jlong>(shellow_engine_live_shell_event_revision(engineFromHandle(handle)));
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeSetRendererOverlayJson(
    JNIEnv *env,
    jobject,
    jlong handle,
    jstring overlayJson
) {
    const std::string overlay = readString(env, overlayJson);
    return takeJson(
        env,
        shellow_engine_set_renderer_overlay_json(engineFromHandle(handle), overlay.c_str())
    );
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeAttachAndroidNativeWindowJson(
    JNIEnv *env,
    jobject,
    jlong handle,
    jlong rawHandle,
    jint widthPx,
    jint heightPx
) {
    return takeJson(
        env,
        shellow_engine_attach_android_native_window_json(
            engineFromHandle(handle),
            static_cast<uint64_t>(rawHandle),
            static_cast<uint32_t>(widthPx),
            static_cast<uint32_t>(heightPx)
        )
    );
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeAttachAndroidSurfaceJson(
    JNIEnv *env,
    jobject,
    jlong handle,
    jobject surface,
    jint widthPx,
    jint heightPx
) {
    if (surface == nullptr) {
        return newStringFromUtf8(env, "{\"error\":\"Android Surface was null\"}");
    }

    ANativeWindow *window = ANativeWindow_fromSurface(env, surface);
    if (window == nullptr) {
        return newStringFromUtf8(env, "{\"error\":\"ANativeWindow_fromSurface returned null\"}");
    }

    replaceSurfaceWindow(handle, window);
    return takeJson(
        env,
        shellow_engine_attach_android_native_window_json(
            engineFromHandle(handle),
            reinterpret_cast<uint64_t>(window),
            static_cast<uint32_t>(widthPx),
            static_cast<uint32_t>(heightPx)
        )
    );
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeDetachRendererSurfaceJson(JNIEnv *env, jobject, jlong handle) {
    jstring result = takeJson(env, shellow_engine_detach_renderer_surface_json(engineFromHandle(handle)));
    releaseSurfaceWindow(handle);
    return result;
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeSendCommandJson(
    JNIEnv *env,
    jobject,
    jlong handle,
    jstring input
) {
    std::string inputText = readString(env, input);
    return takeJson(env, shellow_engine_send_command_json(engineFromHandle(handle), inputText.c_str()));
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeSendTerminalInputJson(
    JNIEnv *env,
    jobject,
    jlong handle,
    jstring input
) {
    std::string inputText = readString(env, input);
    return takeJson(env, shellow_engine_send_terminal_input_json(engineFromHandle(handle), inputText.c_str()));
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeResizeTerminalJson(
    JNIEnv *env,
    jobject,
    jlong handle,
    jint cols,
    jint rows
) {
    return takeJson(
        env,
        shellow_engine_resize_terminal_json(
            engineFromHandle(handle),
            static_cast<uint32_t>(cols),
            static_cast<uint32_t>(rows)
        )
    );
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeClearTerminalJson(JNIEnv *env, jobject, jlong handle) {
    return takeJson(env, shellow_engine_clear_terminal_json(engineFromHandle(handle)));
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeResetTerminalJson(JNIEnv *env, jobject, jlong handle) {
    return takeJson(env, shellow_engine_reset_terminal_json(engineFromHandle(handle)));
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeConnectPreviewJson(
    JNIEnv *env,
    jobject,
    jlong handle,
    jstring name,
    jstring host,
    jint port,
    jstring username,
    jstring trustedHostKeySha256,
    jint authKind
) {
    std::string nameText = readString(env, name);
    std::string hostText = readString(env, host);
    std::string usernameText = readString(env, username);
    std::string trustedHostKeySha256Text = readString(env, trustedHostKeySha256);
    return takeJson(
        env,
        shellow_engine_connect_preview_json(
            engineFromHandle(handle),
            nameText.c_str(),
            hostText.c_str(),
            static_cast<uint16_t>(port),
            usernameText.c_str(),
            trustedHostKeySha256Text.c_str(),
            static_cast<uint8_t>(authKind)
        )
    );
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeStartPasswordShellJson(
    JNIEnv *env,
    jobject,
    jlong handle,
    jstring name,
    jstring host,
    jint port,
    jstring username,
    jstring trustedHostKeySha256,
    jstring password
) {
    std::string nameText = readString(env, name);
    std::string hostText = readString(env, host);
    std::string usernameText = readString(env, username);
    std::string trustedHostKeySha256Text = readString(env, trustedHostKeySha256);
    std::string passwordText = readString(env, password);
    return takeJson(
        env,
        shellow_engine_start_password_shell_json(
            engineFromHandle(handle),
            nameText.c_str(),
            hostText.c_str(),
            static_cast<uint16_t>(port),
            usernameText.c_str(),
            trustedHostKeySha256Text.c_str(),
            passwordText.c_str()
        )
    );
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeStartPrivateKeyShellJson(
    JNIEnv *env,
    jobject,
    jlong handle,
    jstring name,
    jstring host,
    jint port,
    jstring username,
    jstring trustedHostKeySha256,
    jstring privateKeyPem,
    jstring passphrase
) {
    std::string nameText = readString(env, name);
    std::string hostText = readString(env, host);
    std::string usernameText = readString(env, username);
    std::string trustedHostKeySha256Text = readString(env, trustedHostKeySha256);
    std::string privateKeyPemText = readString(env, privateKeyPem);
    std::string passphraseText = readString(env, passphrase);
    return takeJson(
        env,
        shellow_engine_start_private_key_shell_json(
            engineFromHandle(handle),
            nameText.c_str(),
            hostText.c_str(),
            static_cast<uint16_t>(port),
            usernameText.c_str(),
            trustedHostKeySha256Text.c_str(),
            privateKeyPemText.c_str(),
            passphraseText.c_str()
        )
    );
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativePollLiveShellJson(JNIEnv *env, jobject, jlong handle) {
    return takeJson(env, shellow_engine_poll_live_shell_json(engineFromHandle(handle)));
}

extern "C" JNIEXPORT jstring JNICALL
Java_xyz_zinglix_shellow_core_ShellowNative_nativeDisconnectLiveShellJson(JNIEnv *env, jobject, jlong handle) {
    return takeJson(env, shellow_engine_disconnect_live_shell_json(engineFromHandle(handle)));
}
