


fn main() {

    println!("cargo::rerun-if-changed=raw_c");

    cc::Build::new()
    .files([
        "raw_c/src/libdoc_1.c",
        "raw_c/src/libdoc_2.c",
    ])
    .include("raw_c/include")
    .include("raw_c/src")
    .compile("libdoc");

}





