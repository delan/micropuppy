use std::env;

fn main() {
    // Since we're in a workspace, the path we pass to the linker must be relative to the workspace,
    // not the crate -- the linker is run in the workspace root, not in the crate. The cargo
    // documentation states that:
    //     In addition to environment variables, the build script’s current directory is the source
    //     directory of the build script’s package.
    // (see https://doc.rust-lang.org/cargo/reference/build-scripts.html#inputs-to-the-build-script)
    // We therefore use our current directory to get a fully-qualified path to the linker script.
    let linker_script = env::current_dir()
        .expect("build script to have a valid current working directory")
        .join("src/linker.ld");
    let linker_script = linker_script
        .to_str()
        .expect("linker script path to be valid");

    println!("cargo:rerun-if-changed={linker_script}");
    println!("cargo:rustc-link-arg=-T{linker_script}");
}
