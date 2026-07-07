cmake_minimum_required(VERSION 3.22.1)
project(shellow_android)

add_library(shellow_ffi SHARED IMPORTED)
set_target_properties(
    shellow_ffi
    PROPERTIES
    IMPORTED_LOCATION
    "${CMAKE_SOURCE_DIR}/../jniLibs/${ANDROID_ABI}/libshellow_ffi.so"
    IMPORTED_NO_SONAME TRUE
)

add_library(shellow_jni SHARED shellow_jni.cpp)
target_include_directories(
    shellow_jni
    PRIVATE "${SHELLOW_ROOT}/crates/shellow-ffi/include"
)
target_link_libraries(shellow_jni shellow_ffi android)
