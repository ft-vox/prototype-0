fn main() {
    cc::Build::new()
        .file("c_src/library.c")
        .include("include")
        .compile("library");

    println!("cargo:rustc-link-lib=static=library");
}
