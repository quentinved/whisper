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
}
