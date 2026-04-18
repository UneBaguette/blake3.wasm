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