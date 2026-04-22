use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn hash(value: &[u8]) -> Vec<u8> {
    blake3::hash(value).as_bytes().to_vec()
}

#[wasm_bindgen]
pub fn hash_xof(data: &[u8], out_len: usize) -> Vec<u8> {
    let mut out = vec![0u8; out_len];
    let mut reader = blake3::Hasher::new().update(data).finalize_xof();

    reader.fill(&mut out);

    out
}

#[wasm_bindgen]
pub fn keyed_hash(data: &[u8], key: &[u8]) -> Result<Vec<u8>, JsError> {
    let key: &[u8; 32] = key
        .try_into()
        .map_err(|_| JsError::new("key must be exactly 32 bytes"))?;
    Ok(blake3::keyed_hash(key, data).as_bytes().to_vec())
}

#[wasm_bindgen]
pub fn derive_key(context: &str, key_material: &[u8]) -> Vec<u8> {
    blake3::derive_key(context, key_material).to_vec()
}

#[wasm_bindgen]
pub struct Hasher(blake3::Hasher);

#[wasm_bindgen]
impl Hasher {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Hasher(blake3::Hasher::new())
    }

    pub fn new_keyed(key: &[u8]) -> Result<Hasher, JsError> {
        let key: &[u8; 32] = key
            .try_into()
            .map_err(|_| JsError::new("key must be exactly 32 bytes"))?;
        Ok(Hasher(blake3::Hasher::new_keyed(key)))
    }

    pub fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    pub fn finalize(&self) -> Vec<u8> {
        self.0.finalize().as_bytes().to_vec()
    }

    pub fn finalize_xof(&self, out_len: usize) -> Vec<u8> {
        let mut out = vec![0u8; out_len];
        self.0.finalize_xof().fill(&mut out);
        out
    }

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
}

#[cfg(target_arch = "wasm32")]
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