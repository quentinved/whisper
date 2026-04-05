#[cfg(any(test, feature = "test-utils"))]
pub mod mocks {
    use crate::contracts::repositories::managed_secret_repository::{
        ManagedSecretRepository, ManagedSecretRepositoryError,
    };
    use crate::entities::managed_secret::ManagedSecret;
    use crate::values_object::shared_secret::secret_id::SecretId;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    pub struct MockManagedSecretRepository {
        pub secrets: Arc<Mutex<HashMap<SecretId, ManagedSecret>>>,
    }

    impl Default for MockManagedSecretRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockManagedSecretRepository {
        pub fn new() -> Self {
            Self {
                secrets: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    impl ManagedSecretRepository for MockManagedSecretRepository {
        async fn upsert(
            &self,
            secret: ManagedSecret,
        ) -> Result<bool, ManagedSecretRepositoryError> {
            let mut map = self.secrets.lock().unwrap();
            let is_insert = !map.contains_key(&secret.id());
            let id = secret.id();
            map.insert(id, secret);
            Ok(is_insert)
        }

        async fn get_by_id(
            &self,
            id: &SecretId,
        ) -> Result<Option<ManagedSecret>, ManagedSecretRepositoryError> {
            Ok(self.secrets.lock().unwrap().get(id).cloned())
        }

        async fn pull_by_id(
            &self,
            id: &SecretId,
        ) -> Result<Option<ManagedSecret>, ManagedSecretRepositoryError> {
            let mut map = self.secrets.lock().unwrap();
            if let Some(secret) = map.get_mut(id) {
                secret.set_last_pulled_at(chrono::Utc::now());
                Ok(Some(secret.clone()))
            } else {
                Ok(None)
            }
        }

        async fn delete_by_id(&self, id: &SecretId) -> Result<(), ManagedSecretRepositoryError> {
            self.secrets.lock().unwrap().remove(id);
            Ok(())
        }

        async fn get_auth_token_hash(
            &self,
            id: &SecretId,
        ) -> Result<Option<Option<String>>, ManagedSecretRepositoryError> {
            let map = self.secrets.lock().unwrap();
            match map.get(id) {
                Some(s) => Ok(Some(s.auth_token_hash().map(String::from))),
                None => Ok(None),
            }
        }
    }
}
