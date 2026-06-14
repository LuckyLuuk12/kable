use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use std::path::PathBuf;

mod config;
mod entry;
mod hashing;
mod locks;
mod runtime;
mod storage;
// Re-exporting the public API of the cache module
#[doc(hidden)]
pub use runtime::__macro_get_or_compute;
pub use runtime::get_or_compute;
