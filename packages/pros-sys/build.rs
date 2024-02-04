fn main() {
    if cfg!(not(feature = "no-link")) {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

        println!("cargo:rustc-link-search=native={manifest_dir}/link");
        println!("cargo:rustc-link-lib=static=pros");
    }
}
