extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/lookup3.c")
        .compile("liblookup3.a");
}
