use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::str::from_utf8;

fn main() {

    let header_includes = make_print("PROJECT_PATH").split_whitespace().map(|path| {
        format!("-I../core/{}", &path[2..])
    }).collect::<Vec<String>>();

    let bindings = bindgen::Builder::default()
        .clang_arg("--target=armv7a-none-eabi")
        .clang_args(header_includes)
        .use_core()
        .ctypes_prefix("cty")
        .header("../core/bindings.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

// Print a variable from the Makefile
fn make_print(variable: &str) -> String {
    let cmd = Command::new("sh")
        .current_dir("../core")
        .arg("-c")
        .arg(format!("make print-{}", variable))
        .output()
        .expect("failed to execute process");
    if !cmd.status.success() { panic!("Makefile command failed") };
    from_utf8(&cmd.stdout).expect("Failed to read utf8").lines().next().expect("No line").to_owned()
}
