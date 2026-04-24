use wasm_bindgen::prelude::*;

/// Hash data and return a 32-byte BLAKE3 digest.
#[wasm_bindgen]
pub fn hash(value: &[u8]) -> Vec<u8> {
    blake3::hash(value).as_bytes().to_vec()
}

/// Hash data with variable-length output (XOF mode).
/// Returns `out_len` bytes of BLAKE3 extended output.
#[wasm_bindgen]
pub fn hash_xof(data: &[u8], out_len: usize) -> Vec<u8> {
    let mut out = vec![0u8; out_len];
    let mut reader = blake3::Hasher::new().update(data).finalize_xof();

    reader.fill(&mut out);

    out
}

/// Compute a keyed BLAKE3 hash (MAC). Key must be exactly 32 bytes.
/// Throws if the key length is wrong.
#[wasm_bindgen]
pub fn keyed_hash(data: &[u8], key: &[u8]) -> Result<Vec<u8>, JsError> {
    let key: &[u8; 32] = key
        .try_into()
        .map_err(|_| JsError::new("key must be exactly 32 bytes"))?;

    Ok(blake3::keyed_hash(key, data).as_bytes().to_vec())
}

/// Derive a 32-byte key from a context string and key material.
/// Context should be a hardcoded, globally unique, application-specific string.
#[wasm_bindgen]
pub fn derive_key(context: &str, key_material: &[u8]) -> Vec<u8> {
    blake3::derive_key(context, key_material).to_vec()
}

/// Incremental BLAKE3 hasher for streaming data.
///
/// ```js
/// const hasher = new Hasher();
/// hasher.update(chunk1);
/// hasher.update(chunk2);
/// const digest = hasher.finalize(); // 32 bytes
/// ```
#[wasm_bindgen]
pub struct Hasher(blake3::Hasher);

#[wasm_bindgen]
impl Hasher {
    /// Create a new hasher for unkeyed hashing.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Hasher(blake3::Hasher::new())
    }

    /// Create a keyed hasher (MAC mode). Key must be exactly 32 bytes.
    /// Throws if the key length is wrong.
    pub fn new_keyed(key: &[u8]) -> Result<Hasher, JsError> {
        let key: &[u8; 32] = key
            .try_into()
            .map_err(|_| JsError::new("key must be exactly 32 bytes"))?;

        Ok(Hasher(blake3::Hasher::new_keyed(key)))
    }

    /// Create a hasher in derive-key mode.
    /// Context should be a hardcoded, globally unique, application-specific string.
    /// Feed key material via `update()`, then call `finalize()`.
    pub fn new_derive_key(context: &str) -> Hasher {
        Hasher(blake3::Hasher::new_derive_key(context))
    }

    /// Feed data into the hasher. Can be called multiple times for streaming.
    pub fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    /// Return the 32-byte hash digest. Non-destructive. Can be called multiple times.
    pub fn finalize(&self) -> Vec<u8> {
        self.0.finalize().as_bytes().to_vec()
    }

    /// Return `out_len` bytes of extended output (XOF mode). Non-destructive.
    pub fn finalize_xof(&self, out_len: usize) -> Vec<u8> {
        let mut out = vec![0u8; out_len];
        self.0.finalize_xof().fill(&mut out);

        out
    }

    /// Finalize the hash and reset the hasher in one call.
    /// Useful for hashing multiple inputs sequentially without creating new instances.
    pub fn finalize_and_reset(&mut self) -> Vec<u8> {
        let out = self.0.finalize().as_bytes().to_vec();
        self.0.reset();

        out
    }

    /// Reset the hasher to its initial state. Preserves the mode (keyed/derive-key).
    pub fn reset(&mut self) {
        self.0.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_deterministic() {
        let a = hash(b"hello");
        let b = hash(b"hello");
        assert_eq!(a, b);
    }

    #[test]
    fn test_hash_length() {
        assert_eq!(hash(b"hello").len(), 32);
    }

    #[test]
    fn test_hash_xof_length() {
        assert_eq!(hash_xof(b"hello", 64).len(), 64);
        assert_eq!(hash_xof(b"hello", 16).len(), 16);
    }

    #[test]
    fn test_keyed_hash_valid() {
        let key = [0u8; 32];
        assert!(keyed_hash(b"hello", &key).is_ok());
    }

    #[test]
    fn test_derive_key() {
        let a = derive_key("ctx", b"material");
        let b = derive_key("ctx", b"material");
        assert_eq!(a, b);
        assert_eq!(a.len(), 32);

        let c = derive_key("other", b"material");
        assert_ne!(a, c);
    }

    #[test]
    fn test_hasher_streaming() {
        let oneshot = hash(b"helloworld");
        let mut h = Hasher::new();
        h.update(b"hello");
        h.update(b"world");
        assert_eq!(h.finalize(), oneshot);
    }

    #[test]
    fn test_hasher_derive_key() {
        let oneshot = derive_key("my-ctx", b"material");
        let mut h = Hasher::new_derive_key("my-ctx");
        h.update(b"material");
        assert_eq!(h.finalize(), oneshot);
    }

    #[test]
    fn test_hasher_finalize_and_reset() {
        let mut h = Hasher::new();
        h.update(b"hello");
        let first = h.finalize_and_reset();
        assert_eq!(first, hash(b"hello"));

        // After reset, hashing different data should produce different output
        h.update(b"world");
        let second = h.finalize();
        assert_eq!(second, hash(b"world"));
        assert_ne!(first, second);
    }

    #[test]
    fn test_keyed_hash_matches_streaming() {
        let key = [42u8; 32];
        let oneshot = keyed_hash(b"hello world", &key).unwrap();
        let mut h = Hasher::new_keyed(&key).unwrap();
        h.update(b"hello ");
        h.update(b"world");
        assert_eq!(h.finalize(), oneshot);
    }

    #[test]
    fn test_hasher_finalize_xof() {
        let oneshot = hash_xof(b"hello", 64);
        let mut h = Hasher::new();
        h.update(b"hello");
        assert_eq!(h.finalize_xof(64), oneshot);
    }

    #[test]
    fn test_hasher_finalize_xof_prefix_matches_hash() {
        // First 32 bytes of XOF output should equal the standard hash
        let standard = hash(b"test data");
        let xof = hash_xof(b"test data", 64);
        assert_eq!(&xof[..32], standard.as_slice());
    }

    #[test]
    fn test_hasher_reset() {
        let mut h = Hasher::new();
        h.update(b"junk");
        h.reset();
        h.update(b"hello");
        assert_eq!(h.finalize(), hash(b"hello"));
    }

    #[test]
    fn test_hasher_keyed() {
        let key = [1u8; 32];
        let h = Hasher::new_keyed(&key).unwrap();
        assert_ne!(h.finalize(), hash(b"").as_slice());
    }

    // BLAKE3 official test vectors (from BLAKE3 spec)
    // Reference: https://github.com/BLAKE3-team/BLAKE3/blob/master/test_vectors/test_vectors.json
    // Input: 0x00..0xfa repeating pattern, first N bytes

    fn test_input(len: usize) -> Vec<u8> {
        (0..len).map(|i| (i % 251) as u8).collect()
    }

    #[test]
    fn test_vector_empty() {
        let out = hash(&[]);
        let expected =
            hex::decode("af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262")
                .unwrap();
        assert_eq!(out, expected);
    }

    #[test]
    fn test_vector_1_byte() {
        let input = test_input(1);
        let out = hash(&input);
        let expected =
            hex::decode("2d3adedff11b61f14c886e35afa036736dcd87a74d27b5c1510225d0f592e213")
                .unwrap();
        assert_eq!(out, expected);
    }

    #[test]
    fn test_vector_1025_bytes() {
        let input = test_input(1025);
        let out = hash(&input);
        let expected =
            hex::decode("d00278ae47eb27b34faecf67b4fe263f82d5412916c1ffd97c8cb7fb814b8444")
                .unwrap();
        assert_eq!(out, expected);
    }

    #[test]
    fn test_vector_keyed_empty() {
        let key = b"whats the Elvish word for friend";
        let out = keyed_hash(&[], key).unwrap();
        let expected =
            hex::decode("92b2b75604ed3c761f9d6f62392c8a9227ad0ea3f09573e783f1498a4ed60d26")
                .unwrap();
        assert_eq!(out, expected);
    }
}

#[cfg(all(target_arch = "wasm32", test))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_keyed_hash_bad_key() {
        assert!(keyed_hash(b"hello", &[0u8; 16]).is_err());
    }

    #[wasm_bindgen_test]
    fn test_hasher_keyed_bad_key() {
        assert!(Hasher::new_keyed(&[0u8; 10]).is_err());
    }
}
