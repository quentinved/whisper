pub mod managed_secret_steps;
pub mod shared_secret_steps;

use cucumber::World;
use whisper_core::commands::managed_secret::test_utils::mocks::MockManagedSecretRepository;
use whisper_core::commands::shared_secret::test_utils::mocks::{
    MockEncryption, MockSharedSecretRepository,
};
use whisper_core::values_object::shared_secret::secret_id::SecretId;

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct WhisperWorld {
    // Mock dependencies
    pub encryption: MockEncryption,
    pub shared_secret_repo: MockSharedSecretRepository,
    pub managed_secret_repo: MockManagedSecretRepository,

    // SharedSecret state
    pub secret_input: Option<String>,
    pub secret_expiration_hours: Option<i64>,
    pub secret_self_destruct: Option<bool>,
    pub created_secret_id: Option<SecretId>,
    pub retrieved_secret: Option<String>,
    pub self_destruct_flag: Option<bool>,
    pub deleted_count: Option<u64>,

    // ManagedSecret state
    pub managed_secret_id: Option<SecretId>,
    pub managed_payload: Option<Vec<u8>>,
    pub retrieved_payload: Option<Vec<u8>>,
    pub upsert_was_insert: Option<bool>,

    // Error capture
    pub last_error: Option<String>,
}

impl WhisperWorld {
    fn new() -> Self {
        Self {
            encryption: MockEncryption,
            shared_secret_repo: MockSharedSecretRepository::new(),
            managed_secret_repo: MockManagedSecretRepository::new(),

            secret_input: None,
            secret_expiration_hours: None,
            secret_self_destruct: None,
            created_secret_id: None,
            retrieved_secret: None,
            self_destruct_flag: None,
            deleted_count: None,

            managed_secret_id: None,
            managed_payload: None,
            retrieved_payload: None,
            upsert_was_insert: None,

            last_error: None,
        }
    }
}
