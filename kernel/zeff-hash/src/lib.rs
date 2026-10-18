//! Content hashing (Engine and Project Specification, section 8.3, point 10, and section 8.7,
//! "Hashing"): every input file, cached copy, output file and fetched response is hashed with
//! BLAKE3, and the hash is stored with the algorithm's name, so a record stays readable if the
//! algorithm ever changes. Until the run record exists (0.11.0) the hash of an imported file is
//! kept in the `source` metadata of its columns.

/// The algorithm's name, as stored beside every hash.
pub const ALGORITHM: &str = "blake3";

/// The BLAKE3 hash of `bytes` as 64 lowercase hexadecimal digits.
pub fn hash_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// The hash with its algorithm's name, `blake3:<64 hex digits>`, as the run record will store it.
pub fn labelled(bytes: &[u8]) -> String {
    format!("{ALGORITHM}:{}", hash_hex(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vectors_of_the_blake3_specification() {
        // The empty input and the one-byte input of the official test vectors.
        assert_eq!(
            hash_hex(b""),
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
        assert_eq!(
            hash_hex(&[0u8]),
            "2d3adedff11b61f14c886e35afa036736dcd87a74d27b5c1510225d0f592e213"
        );
        assert_eq!(labelled(b"").len(), "blake3:".len() + 64);
        assert!(labelled(b"abc").starts_with("blake3:"));
        assert_ne!(hash_hex(b"abc"), hash_hex(b"abd"));
    }
}
