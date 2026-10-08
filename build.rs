fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("TARGET").as_deref() == Ok("wasm32-unknown-unknown") {
        // Browser audio and visibility imports are supplied by the local JS runtime.
        println!("cargo:rustc-link-arg-bin=elite=--import-undefined");
    }
}
