use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/pa_ctx.c");
    println!("cargo:rerun-if-changed=src/virtual_device.c");
    println!("cargo:rerun-if-changed=src/stream.c");

    cc::Build::new()
        .flag("-Wall")
        .flag("-Wextra")
        .flag("-pedantic")
        .include("../../ffi/include")
        .file("src/pa_ctx.c")
        .file("src/virtual_device.c")
        .file("src/stream.c")
        .compile("ffone_c_pa");

    pkg_config::Config::new()
        .cargo_metadata(true)
        .probe("libpulse")?;

    Ok(())
}
