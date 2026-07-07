fn main() {
    println!("cargo:rerun-if-env-changed=LIBGHOSTTY_VT_SYS_OPTIMIZE");
    println!("cargo:rerun-if-env-changed=GHOSTTY_SOURCE_DIR");
    println!("cargo:rerun-if-env-changed=GHOSTTY_ZIG_SYSTEM_DIR");
    println!("cargo:rerun-if-env-changed=ZIG_GLOBAL_CACHE_DIR");
}
