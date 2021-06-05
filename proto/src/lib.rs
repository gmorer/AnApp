pub use prost;
pub use std::sync::Arc;

#[cfg(feature = "server")]
pub mod server {
    #[path = "grpc.hello.rs"]
    pub mod hello;
}


#[cfg(feature = "client")]
pub mod client {
    #[path = "grpc.hello.rs"]
    pub mod hello;
}
