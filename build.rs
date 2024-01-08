use std::env;
use std::path::Path;

fn main() {
    let dest = env::var("OUT_DIR").unwrap();

    println!("cargo:rerun-if-changed=src/objcpp/");
    let src = ["src/objcpp/renderer.mm"];
    cc::Build::new().cpp(true).flag("-std=c++17").files(src.iter()).compile("objcpp");
    let bindings = bindgen::Builder::default()
        .header("src/objcpp/wrapper.h")
        .clang_arg("-xobjective-c++")
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(Path::new(&dest).join("objcpp_bindings.rs"))
        .expect("Couldn't write bindings!");
}
