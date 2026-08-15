//! The nbsp crate build script

fn main() {
    memory_serve::load_directory("./assets");

    println!("cargo:rerun-if-changed=migrations");
}
