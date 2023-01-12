use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    println!(
        "cargo:rustc-link-search=native={}/../../../../../intel-cmt-cat/lib",
        out_dir
    );
    println!("cargo:rustc-link-lib=dylib=pqos");
}
