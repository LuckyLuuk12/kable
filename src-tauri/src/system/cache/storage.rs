use bincode::config::standard;
use bincode::serde::encode_to_vec;
use serde::Serialize;

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
