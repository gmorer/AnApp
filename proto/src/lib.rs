pub use prost;
pub use std::sync::Arc;

#[cfg(feature = "server")]
pub mod server {
    #[path = "grpc.auth.rs"]
    pub mod auth;
}

#[cfg(feature = "client")]
pub mod client {
    #[path = "grpc.auth.rs"]
    pub mod auth;
}
