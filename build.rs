use std::env;
use std::path::Path;

fn main() {
    let dest = env::var("OUT_DIR").unwrap();

    println!("cargo:rerun-if-changed=src/cpp/");
    let src = ["src/cpp/renderer.cc"];
    cc::Build::new().cpp(true).flag("-std=c++17").files(src.iter()).compile("mybar");
    let bindings = bindgen::Builder::default()
        .header("src/cpp/wrapper.h")
        .clang_arg("-xc++")
        .clang_arg("-libc++")
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(Path::new(&dest).join("cpp_bindings.rs"))
        .expect("Couldn't write bindings!");
}
