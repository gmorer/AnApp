const SCHEMAS: [&str; 2] = ["schemas/auth.proto", "schemas/user.proto"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=schemas");
    #[cfg(feature = "server")]
    {
        std::fs::create_dir("src/server").or_else(|e| match e.kind() {
            std::io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(e),
        })?;
        tonic_build::configure()
            .out_dir("src/server")
            .build_server(true)
            .build_client(false)
            .compile(&SCHEMAS, &["schemas"])?;
    }
    #[cfg(feature = "client")]
    {
        std::fs::create_dir("src/client").or_else(|e| match e.kind() {
            std::io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(e),
        })?;
        tonic_build::configure()
            .out_dir("src/client")
            .build_server(false)
            .build_client(true)
            .compile(&SCHEMAS, &["schemas"])?;
    }
    Ok(())
}
