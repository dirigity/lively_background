fn main() {
    cxx_build::bridge("src/main.rs")
        .compile("cxx-demo");
}