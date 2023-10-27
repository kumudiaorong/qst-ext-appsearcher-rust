extern crate tonic_build;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        // .out_dir("src/comm")
        .compile(
            &["defs.proto", "daemon.proto", "extension.proto"],
            &["qst-grpc/src"],
        )?;
    Ok(())
}
