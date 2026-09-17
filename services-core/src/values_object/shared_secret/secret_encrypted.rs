pub const NONCE_SIZE: usize = 12;

#[derive(Clone, Debug)]
pub struct SecretEncrypted {
    nonce: [u8; NONCE_SIZE],
    cypher: Vec<u8>,
}

impl SecretEncrypted {
    pub fn new(nonce: [u8; NONCE_SIZE], cypher: Vec<u8>) -> Self {
        Self { nonce, cypher }
    }

    pub fn nonce(&self) -> &[u8; NONCE_SIZE] {
        &self.nonce
    }

    pub fn cypher(&self) -> &[u8] {
        &self.cypher
    }

    pub fn into_parts(self) -> ([u8; NONCE_SIZE], Vec<u8>) {
        (self.nonce, self.cypher)
    }

    /// Wire payload: `nonce ‖ ciphertext`. The cross-surface zero-knowledge format.
    pub fn into_payload(self) -> Vec<u8> {
        let mut p = Vec::with_capacity(NONCE_SIZE + self.cypher.len());
        p.extend_from_slice(&self.nonce);
        p.extend(self.cypher);
        p
    }

    /// Parse `nonce[12] ‖ ciphertext`; needs at least nonce + 1 byte.
    pub fn try_from_payload(payload: &[u8]) -> Option<Self> {
        if payload.len() < NONCE_SIZE + 1 {
            return None;
        }
        let nonce: [u8; NONCE_SIZE] = payload[..NONCE_SIZE].try_into().ok()?;
        Some(Self::new(nonce, payload[NONCE_SIZE..].to_vec()))
    }
}
