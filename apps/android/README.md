# Shellow Android

This is the native Android shell for Shellow. It uses Jetpack Compose for the app UI and loads the shared Rust core through JNI.

## Build

From the repository root:

```sh
./scripts/build-android-rust.sh
cd apps/android
./gradlew :app:assembleDebug
```

The debug APK is generated at:

```text
app/build/outputs/apk/debug/app-debug.apk
```

## Run

Connect a device and launch the APK:

```sh
adb connect <host>:<port>
android run --device=<host>:<port> --apks=app/build/outputs/apk/debug/app-debug.apk --activity=xyz.zinglix.shellow.MainActivity
```

The app packages:

- `libshellow_ffi.so`: Rust core and C ABI.
- `libshellow_jni.so`: JNI wrapper used by Kotlin.
