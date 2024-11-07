


fn main() {

    println!("cargo::rerun-if-changed=raw_c");

    cc::Build::new()
    .files([
        "raw_c/libdoc_1.c",
        "raw_c/libdoc_2.c",
    ]).compile("libdoc");

}





