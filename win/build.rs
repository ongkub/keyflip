fn main() {
    println!("cargo:rerun-if-changed=keyflip.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") && cfg!(windows) {
        winresource::WindowsResource::new()
            .set_icon("keyflip.ico")
            .compile()
            .expect("failed to compile Windows resources");
    }
}
