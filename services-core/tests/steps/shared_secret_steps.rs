use crate::steps::WhisperWorld;
use chrono::{Duration, Utc};
use cucumber::{given, then, when};
use whisper_core::commands::shared_secret::create_secret::CreateSecret;
use whisper_core::commands::shared_secret::delete_expired_secrets::DeleteExpiredSecrets;
use whisper_core::commands::shared_secret::get_secret_by_id::GetSecretById;
use whisper_core::contracts::repositories::shared_secret_repository::SharedSecretRepository;
use whisper_core::entities::shared_secret::SharedSecret;
use whisper_core::services::secret_encryption::SecretEncryption;
use whisper_core::values_object::shared_secret::secret_encrypted::SecretEncrypted;
use whisper_core::values_object::shared_secret::secret_expiration::SecretExpiration;
use whisper_core::values_object::shared_secret::secret_id::SecretId;

// ── Given ──

#[given(expr = "a secret {string} with expiration in {int} hours and self-destruct enabled")]
async fn given_secret_with_self_destruct(world: &mut WhisperWorld, secret: String, hours: i64) {
    world.secret_input = Some(secret);
    world.secret_expiration_hours = Some(hours);
    world.secret_self_destruct = Some(true);
}

#[given(expr = "a secret {string} with expiration in {int} hours and self-destruct disabled")]
async fn given_secret_without_self_destruct(world: &mut WhisperWorld, secret: String, hours: i64) {
    world.secret_input = Some(secret);
    world.secret_expiration_hours = Some(hours);
    world.secret_self_destruct = Some(false);
}

#[given(expr = "a secret larger than 64 KB with expiration in {int} hours")]
async fn given_oversized_secret(world: &mut WhisperWorld, hours: i64) {
    world.secret_input = Some("x".repeat(64 * 1024 + 1));
    world.secret_expiration_hours = Some(hours);
    world.secret_self_destruct = Some(false);
}

#[given(expr = "a stored secret {string} with self-destruct disabled")]
async fn given_stored_secret_no_self_destruct(world: &mut WhisperWorld, secret: String) {
    let id = SecretId::generate();
    let future_time = (Utc::now() + Duration::hours(1)).timestamp();
    let expiration = SecretExpiration::try_from(future_time).unwrap();
    let encrypted = world.encryption.encrypt_secret(&secret).unwrap();
    let shared_secret = SharedSecret::new(id, encrypted, expiration, false);
    world.shared_secret_repo.insert(shared_secret);
    world.created_secret_id = Some(id);
}

#[given(expr = "a stored secret {string} with self-destruct enabled")]
async fn given_stored_secret_with_self_destruct(world: &mut WhisperWorld, secret: String) {
    let id = SecretId::generate();
    let future_time = (Utc::now() + Duration::hours(1)).timestamp();
    let expiration = SecretExpiration::try_from(future_time).unwrap();
    let encrypted = world.encryption.encrypt_secret(&secret).unwrap();
    let shared_secret = SharedSecret::new(id, encrypted, expiration, true);
    world.shared_secret_repo.insert(shared_secret);
    world.created_secret_id = Some(id);
}

#[given("a random secret ID")]
async fn given_random_secret_id(world: &mut WhisperWorld) {
    world.created_secret_id = Some(SecretId::generate());
}

#[given("a stored secret that has expired")]
async fn given_expired_secret(world: &mut WhisperWorld) {
    let id = SecretId::generate();
    let expired = SharedSecret::new(
        id,
        SecretEncrypted::new([0u8; 12], vec![1, 2, 3]),
        SecretExpiration::from_datetime(Utc::now() - Duration::hours(1)),
        false,
    );
    world.shared_secret_repo.insert(expired);
}

#[given("a stored secret that has not expired")]
async fn given_valid_secret(world: &mut WhisperWorld) {
    let id = SecretId::generate();
    let future_time = (Utc::now() + Duration::hours(1)).timestamp();
    let expiration = SecretExpiration::try_from(future_time).unwrap();
    let valid = SharedSecret::new(
        id,
        SecretEncrypted::new([1u8; 12], vec![4, 5, 6]),
        expiration,
        false,
    );
    world.shared_secret_repo.insert(valid);
}

// ── When ──

#[when("I create the secret")]
async fn when_create_secret(world: &mut WhisperWorld) {
    let secret = world.secret_input.take().unwrap();
    let hours = world.secret_expiration_hours.take().unwrap();
    let self_destruct = world.secret_self_destruct.take().unwrap();
    let future_time = (Utc::now() + Duration::hours(hours)).timestamp();
    let expiration = SecretExpiration::try_from(future_time).unwrap();

    let command = CreateSecret::new(secret, expiration, self_destruct).unwrap();
    let result = command
        .handle(&world.encryption, &world.shared_secret_repo)
        .await;

    match result {
        Ok(id) => {
            world.created_secret_id = Some(id);
            world.self_destruct_flag = world
                .shared_secret_repo
                .get_by_id(&id)
                .await
                .unwrap()
                .map(|s: SharedSecret| s.self_destruct());
        }
        Err(e) => world.last_error = Some(format!("{e}")),
    }
}

#[when("I try to create the secret")]
async fn when_try_create_secret(world: &mut WhisperWorld) {
    let secret = world.secret_input.take().unwrap();
    let hours = world.secret_expiration_hours.take().unwrap();
    let self_destruct = world.secret_self_destruct.take().unwrap();
    let future_time = (Utc::now() + Duration::hours(hours)).timestamp();
    let expiration = SecretExpiration::try_from(future_time).unwrap();

    match CreateSecret::new(secret, expiration, self_destruct) {
        Ok(command) => match command
            .handle(&world.encryption, &world.shared_secret_repo)
            .await
        {
            Ok(id) => world.created_secret_id = Some(id),
            Err(e) => world.last_error = Some(format!("{e}")),
        },
        Err(e) => world.last_error = Some(format!("{e}")),
    }
}

#[when("I retrieve the secret by its ID")]
async fn when_get_secret_by_id(world: &mut WhisperWorld) {
    let id = world.created_secret_id.unwrap();
    let command = GetSecretById::new(id);
    let result = command
        .handle(&world.encryption, &world.shared_secret_repo)
        .await;

    match result {
        Ok(Some((secret, self_destruct))) => {
            world.retrieved_secret = Some(secret);
            world.self_destruct_flag = Some(self_destruct);
        }
        Ok(None) => {
            world.retrieved_secret = None;
        }
        Err(e) => world.last_error = Some(format!("{e}")),
    }
}

#[when("I delete expired secrets")]
async fn when_delete_expired(world: &mut WhisperWorld) {
    let command = DeleteExpiredSecrets::new();
    let result = command.handle(&world.shared_secret_repo).await;

    match result {
        Ok(count) => world.deleted_count = Some(count),
        Err(e) => world.last_error = Some(format!("{e}")),
    }
}

// ── Then ──

#[then("the secret should be stored successfully")]
async fn then_secret_stored(world: &mut WhisperWorld) {
    assert!(
        world.last_error.is_none(),
        "unexpected error: {:?}",
        world.last_error
    );
    let id = world.created_secret_id.unwrap();
    let saved = world.shared_secret_repo.get_by_id(&id).await.unwrap();
    assert!(saved.is_some(), "secret was not found in repository");
}

#[then("self-destruct should be disabled")]
async fn then_self_destruct_disabled(world: &mut WhisperWorld) {
    assert_eq!(world.self_destruct_flag, Some(false));
}

#[then("self-destruct should be enabled")]
async fn then_self_destruct_enabled(world: &mut WhisperWorld) {
    assert_eq!(world.self_destruct_flag, Some(true));
}

#[then(expr = "I should get a {string} error")]
async fn then_should_get_error(world: &mut WhisperWorld, error_substring: String) {
    let error = world
        .last_error
        .as_ref()
        .expect("expected an error but got none");
    assert!(
        error.contains(&error_substring),
        "expected error containing '{}', got '{}'",
        error_substring,
        error
    );
}

#[then(expr = "I should see the decrypted value {string}")]
async fn then_see_decrypted_value(world: &mut WhisperWorld, expected: String) {
    assert_eq!(world.retrieved_secret.as_deref(), Some(expected.as_str()));
}

#[then("the secret should not be found")]
async fn then_secret_not_found(world: &mut WhisperWorld) {
    assert!(
        world.retrieved_secret.is_none(),
        "expected secret to not be found"
    );
}

#[then(expr = "{int} expired secret(s) should be deleted")]
async fn then_n_expired_deleted(world: &mut WhisperWorld, expected: u64) {
    assert_eq!(world.deleted_count, Some(expected));
}

#[then(expr = "{int} secret(s) should remain")]
async fn then_n_secrets_remain(world: &mut WhisperWorld, expected: usize) {
    assert_eq!(world.shared_secret_repo.count(), expected);
}
