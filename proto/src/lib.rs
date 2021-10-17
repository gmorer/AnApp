pub use prost;
pub use std::sync::Arc;

#[cfg(feature = "server")]
pub mod server {
    #[path = "grpc.auth.rs"]
    pub mod auth;
    #[path = "grpc.user.rs"]
    pub mod user;
}

#[cfg(feature = "client")]
pub mod client {
    #[path = "grpc.auth.rs"]
    pub mod auth;
    #[path = "grpc.user.rs"]
    pub mod user;
}
