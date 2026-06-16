mod config;
mod entry;
mod error;
mod hashing;
mod locks;
mod runtime;
mod storage;
// Re-exporting the public API of the cache module
//TODO: Right now this caching system is complete LLM garbage and not usable, in the future look at this and make it work!
#[doc(hidden)]
pub use runtime::__macro_get_or_compute;
pub use runtime::get_or_compute;
