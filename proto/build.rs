const DEST: &str = "src/grpc";

#[cfg(feature = "server")]
const BUILD_SERVER: bool = true;
#[cfg(not(feature = "server"))]
const BUILD_SERVER: bool = false;

#[cfg(feature = "client")]
const BUILD_CLIENT: bool = true;
#[cfg(not(feature = "client"))]
const BUILD_CLIENT: bool = false;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	std::fs::create_dir(DEST).or_else(|e| match e.kind() {
		std::io::ErrorKind::AlreadyExists => Ok(()),
		_ => Err(e)
	})?;
	tonic_build::configure()
		.out_dir(DEST)
		.build_server(BUILD_SERVER)
		.build_client(BUILD_CLIENT)
		.compile(
			&["schemas/hello.proto"],
			&["schemas"],
		)?;
    Ok(())
}