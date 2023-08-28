use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    println!("cargo:rerun-if-changed=src/util.c");
    println!("cargo:rerun-if-changed=src/rc.c");
    println!("cargo:rerun-if-changed=include/*");

    cbindgen::Builder::new()
        .with_config(cbindgen::Config::from_file("cbindgen.toml")?)
        .with_src("../core/src/audio_system/audio.rs")
        .with_src("src/audio_system/audio.rs")
        .with_include_guard("_FFONE_AUDIO_H")
        .with_sys_include("stdint.h")
        .with_sys_include("stddef.h")
        .include_item("RawAudioFormat")
        .include_item("RawAudioBuffer")
        .generate()?
        .write_to_file("include/audio.h");

    cbindgen::Builder::new()
        .with_config(cbindgen::Config::from_file("cbindgen.toml")?)
        .with_src("src/audio_system/queue.rs")
        .with_include_guard("_FFONE_QUEUE_H")
        .with_include("audio.h")
        .with_sys_include("stdlib.h")
        .with_sys_include("stdint.h")
        .with_sys_include("stdbool.h")
        .with_after_include("\ntypedef struct RawAudioQueue RawAudioQueue;")
        .exclude_item("RawAudioQueueRC")
        .generate()?
        .write_to_file("include/queue.h");

    bindgen::Builder::default()
        .header("include/util.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .disable_header_comment()
        .generate_comments(true)
        .allowlist_function("ffone_format_str")
        .allowlist_function("ffone_get_pid")
        .generate()?
        .write_to_file("src/util.rs")?;

    bindgen::Builder::default()
        .header("include/rc.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .disable_header_comment()
        .generate_comments(true)
        .allowlist_function("ffone_rc_alloc")
        .allowlist_function("ffone_rc_alloc0")
        .allowlist_function("ffone_rc_set_dtor")
        .allowlist_function("ffone_rc_ref")
        .allowlist_function("ffone_rc_unref")
        .allowlist_function("ffone_rc_ref_weak")
        .allowlist_function("ffone_rc_unref_weak")
        .allowlist_function("ffone_rc_is_destructed")
        .generate()?
        .write_to_file("src/rc.rs")?;

    cc::Build::new()
        .flag("-Wall")
        .flag("-Wextra")
        .flag("-Wpedantic")
        .include("include/")
        .file("src/util.c")
        .file("src/rc.c")
        .compile("ffone_ffi");

    Ok(())
}
