fn main() {
    cc::Build::new()
        .file("../../c/mod/src/mod.c")
        .include("../../c/TMap/include")
        .include("../../c/mod/include")
        .compile("mod");

    println!("cargo:rustc-link-lib=static=mod");
}
