fn main() {
    cc::Build::new()
        .file("../../c/library/src/library.c")
        .include("../../c/library/include")
        .compile("library");

    println!("cargo:rustc-link-lib=static=library");
}
