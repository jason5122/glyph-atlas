use std::env;
use std::path::Path;

fn main() {
    let dest = env::var("OUT_DIR").unwrap();

    println!("cargo:rerun-if-changed=src/cpp/");

    let src = ["src/cpp/renderer.cc"];
    cc::Build::new().cpp(true).flag("-std=c++17").files(src.iter()).compile("cpp");
    let bindings = bindgen::Builder::default()
        .header("src/cpp/wrapper.h")
        .clang_arg("-xc++")
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(Path::new(&dest).join("cpp_bindings.rs"))
        .expect("Couldn't write bindings!");

    let objc_src = ["src/cpp/hi.m"];
    cc::Build::new().files(objc_src.iter()).compile("objc");
    let objc_bindings = bindgen::Builder::default()
        .header("src/cpp/objc_wrapper.h")
        .clang_arg("-xobjective-c")
        .generate()
        .expect("Unable to generate bindings");
    objc_bindings
        .write_to_file(Path::new(&dest).join("objc_bindings.rs"))
        .expect("Couldn't write bindings!");
}
