fn main() {
    cc::Build::new()
        .file("../../c/TMap/src/TMap.c")
        .include("../../c/TMap/include")
        .compile("tmap");

    println!("cargo:rustc-link-lib=static=tmap");
}
