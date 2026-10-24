mod config;
mod entry;
mod hashing;
mod locks;
pub mod runtime;
mod storage;
// Re-exporting the public API of the cache module

pub use config::initialize;
#[doc(hidden)]
pub use runtime::__macro_get_or_compute;
pub use runtime::get_or_compute;
pub use runtime::{invalidate, invalidate_all, invalidate_group, invalidate_memory, invalidate_no_args, list_entries, CacheEntryInfo};
