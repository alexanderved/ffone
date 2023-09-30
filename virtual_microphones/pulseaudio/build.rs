use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/pa_ctx.*");
    println!("cargo:rerun-if-changed=src/core.*");
    println!("cargo:rerun-if-changed=src/virtual_device.*");
    println!("cargo:rerun-if-changed=src/stream.*");

    cc::Build::new()
        .flag("-Wall")
        .flag("-Wextra")
        .flag("-pedantic")
        .include("../../ffi/include")
        .file("src/pa_ctx.c")
        .file("src/core.c")
        .file("src/virtual_device.c")
        .file("src/stream.c")
        .compile("ffone_c_pa");

    pkg_config::Config::new()
        .cargo_metadata(true)
        .probe("libpulse")?;

    Ok(())
}
