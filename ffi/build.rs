use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    println!("cargo:rerun-if-changed=src/util.c");
    println!("cargo:rerun-if-changed=include/*");

    cbindgen::Builder::new()
        .with_config(cbindgen::Config::from_file("cbindgen.toml")?)
        .with_src("../core/src/audio_system/audio.rs")
        .with_include_guard("_FFONE_AUDIO_H")
        .with_sys_include("stdint.h")
        .include_item("RawAudioFormat")
        .generate()?
        .write_to_file("include/audio.h");

    cbindgen::Builder::new()
        .with_config(cbindgen::Config::from_file("cbindgen.toml")?)
        .with_src("src/audio_system/queue.rs")
        .with_include_guard("_FFONE_QUEUE_H")
        .with_include("audio.h")
        .with_sys_include("stdlib.h")
        .with_sys_include("stdint.h")
        .exclude_item("RawAudioQueueRC")
        .rename_item("CRawAudioQueueRC", "RawAudioQueueRC")
        .generate()?
        .write_to_file("include/queue.h");

    bindgen::Builder::default()
        .header("include/util.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .disable_header_comment()
        .generate_comments(true)
        .generate()?
        .write_to_file("src/util.rs")?;

    cc::Build::new().file("src/util.c").compile("util");

    Ok(())
}
