//! Pinned content-hash helpers for genesis identity.
//!
//! **Pinned algorithm:** BLAKE3, truncated to the first 128 bits (`u128`,
//! little-endian interpretation of the digest prefix). Replay stability depends
//! on this never changing silently.

use blake3::Hasher;

/// Absorb a `u32` as little-endian.
pub(crate) fn update_u32(hasher: &mut Hasher, value: u32) {
    hasher.update(&value.to_le_bytes());
}

/// Absorb an `i32` as little-endian.
pub(crate) fn update_i32(hasher: &mut Hasher, value: i32) {
    hasher.update(&value.to_le_bytes());
}

/// Absorb a length-prefixed UTF-8 string (`u32` LE length + bytes).
pub(crate) fn update_str(hasher: &mut Hasher, s: &str) {
    update_u32(
        hasher,
        u32::try_from(s.len()).expect("string length fits u32"),
    );
    hasher.update(s.as_bytes());
}

/// Finalize a hasher to a 128-bit content id.
pub(crate) fn finalize_u128(hasher: &Hasher) -> u128 {
    let digest = hasher.finalize();
    let bytes = digest.as_bytes();
    u128::from_le_bytes(bytes[..16].try_into().expect("blake3 digest is 32 bytes"))
}
