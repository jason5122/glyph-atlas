use std::env;
use std::fs::File;
use std::path::Path;

use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};

fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(Path::new(&dest).join("gl_bindings.rs")).unwrap();
    Registry::new(
        Api::Gl,
        (3, 3),
        Profile::Core,
        Fallbacks::All,
        ["GL_ARB_blend_func_extended", "GL_KHR_debug"],
    )
    .write_bindings(GlobalGenerator, &mut file)
    .unwrap();

    println!("cargo:rerun-if-changed=src/cpp/");
    let src = ["src/cpp/renderer.cc"];
    cc::Build::new().cpp(true).files(src.iter()).compile("mybar");
    let bindings = bindgen::Builder::default()
        .header("src/cpp/wrapper.h")
        .clang_arg("-xc++")
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(Path::new(&dest).join("cpp_bindings.rs"))
        .expect("Couldn't write bindings!");
}
