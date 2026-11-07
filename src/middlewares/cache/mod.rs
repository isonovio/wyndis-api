mod buffer;
mod kv;
mod layer;
mod service;

pub use buffer::BufferedResponse;
pub use kv::{CachedValue, RequestKey};
pub use layer::CacheLayer;

#[cfg(test)]
#[path = "../../tests/middlewares/cache/tests.rs"]
mod tests;
