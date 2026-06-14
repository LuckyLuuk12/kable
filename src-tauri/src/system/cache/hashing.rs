use bincode::config::standard;
use bincode::serde::encode_to_vec;
use serde::Serialize;
use sha2::{Digest, Sha256};

/// Stable hash of arbitrary serialized input.
///
/// This is the core of the caching system.
/// It ensures identical inputs always map to the same cache file.
pub fn hash_bytes(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hex::encode(hasher.finalize())
}

/// Macro-friendly hashing for function arguments.
///
/// This avoids Debug-based hashing and ensures stability.
///
/// IMPORTANT:
/// - inputs must be serializable via serde
/// - tuple serialization is used for multi-arg functions
pub fn hash_args<T: Serialize>(args: &T) -> String {
    let bytes = encode_to_vec(args, standard()).expect("failed to encode cache key args");

    hash_bytes(&bytes)
}
