pub use prost;
pub use std::sync::Arc;

#[cfg(feature = "server")]
pub mod server {
    #[path = "grpc.users.rs"]
    pub mod users;
}

#[cfg(feature = "client")]
pub mod client {
    #[path = "grpc.users.rs"]
    pub mod users;
}
