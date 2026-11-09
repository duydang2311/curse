use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=vendor/miniaudio/miniaudio.h");
    println!("cargo:rerun-if-changed=vendor/miniaudio/miniaudio.c");

    let bindings = bindgen::Builder::default()
        .header("vendor/miniaudio/miniaudio.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("failed to write bindings");
    cc::Build::new()
        .file("vendor/miniaudio/miniaudio.c")
        .compile("miniaudio");
}
