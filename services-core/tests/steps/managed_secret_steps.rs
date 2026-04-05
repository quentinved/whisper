use crate::steps::WhisperWorld;
use cucumber::{given, then, when};
use whisper_core::commands::managed_secret::delete_managed_secret::DeleteManagedSecret;
use whisper_core::commands::managed_secret::get_managed_secret::GetManagedSecret;
use whisper_core::commands::managed_secret::upsert_managed_secret::UpsertManagedSecret;
use whisper_core::contracts::repositories::managed_secret_repository::ManagedSecretRepository;
use whisper_core::values_object::shared_secret::secret_id::SecretId;

// ── Given ──

#[given(expr = "a managed secret payload {string}")]
async fn given_managed_payload(world: &mut WhisperWorld, payload: String) {
    let id = SecretId::generate();
    world.managed_secret_id = Some(id);
    world.managed_payload = Some(payload.into_bytes());
}

#[given(expr = "an existing managed secret with payload {string}")]
async fn given_existing_managed_secret(world: &mut WhisperWorld, payload: String) {
    let id = SecretId::generate();
    world.managed_secret_id = Some(id);
    let command = UpsertManagedSecret::new(id, payload.into_bytes(), "token_hash".to_string());
    command.handle(&world.managed_secret_repo).await.unwrap();
}

#[given("an empty managed secret payload")]
async fn given_empty_payload(world: &mut WhisperWorld) {
    let id = SecretId::generate();
    world.managed_secret_id = Some(id);
    world.managed_payload = Some(vec![]);
}

#[given("a random secret ID for managed secrets")]
async fn given_random_managed_id(world: &mut WhisperWorld) {
    world.managed_secret_id = Some(SecretId::generate());
}

// ── When ──

#[when("I upsert the managed secret")]
async fn when_upsert(world: &mut WhisperWorld) {
    let id = world.managed_secret_id.unwrap();
    let payload = world.managed_payload.take().unwrap();
    let command = UpsertManagedSecret::new(id, payload, "token_hash".to_string());

    match command.handle(&world.managed_secret_repo).await {
        Ok(is_insert) => world.upsert_was_insert = Some(is_insert),
        Err(e) => world.last_error = Some(format!("{e}")),
    }
}

#[when("I try to upsert the managed secret")]
async fn when_try_upsert(world: &mut WhisperWorld) {
    let id = world.managed_secret_id.unwrap();
    let payload = world.managed_payload.take().unwrap();
    let command = UpsertManagedSecret::new(id, payload, "token_hash".to_string());

    match command.handle(&world.managed_secret_repo).await {
        Ok(is_insert) => world.upsert_was_insert = Some(is_insert),
        Err(e) => world.last_error = Some(format!("{e}")),
    }
}

#[when(expr = "I upsert the managed secret with payload {string}")]
async fn when_upsert_with_payload(world: &mut WhisperWorld, payload: String) {
    let id = world.managed_secret_id.unwrap();
    let command = UpsertManagedSecret::new(id, payload.into_bytes(), "token_hash".to_string());

    match command.handle(&world.managed_secret_repo).await {
        Ok(is_insert) => world.upsert_was_insert = Some(is_insert),
        Err(e) => world.last_error = Some(format!("{e}")),
    }
}

#[when("I retrieve the managed secret by its ID")]
async fn when_get_managed(world: &mut WhisperWorld) {
    let id = world.managed_secret_id.unwrap();
    let command = GetManagedSecret::new(id);

    match command.handle(&world.managed_secret_repo).await {
        Ok(Some(payload)) => world.retrieved_payload = Some(payload),
        Ok(None) => world.retrieved_payload = None,
        Err(e) => world.last_error = Some(format!("{e}")),
    }
}

#[when("I delete the managed secret")]
async fn when_delete_managed(world: &mut WhisperWorld) {
    let id = world.managed_secret_id.unwrap();
    let command = DeleteManagedSecret::new(id);

    match command.handle(&world.managed_secret_repo).await {
        Ok(()) => {}
        Err(e) => world.last_error = Some(format!("{e}")),
    }
}

// ── Then ──

#[then("the upsert should indicate a new insert")]
async fn then_was_insert(world: &mut WhisperWorld) {
    assert_eq!(world.upsert_was_insert, Some(true));
}

#[then("the upsert should indicate an update")]
async fn then_was_update(world: &mut WhisperWorld) {
    assert_eq!(world.upsert_was_insert, Some(false));
}

#[then(expr = "I should see the managed payload {string}")]
async fn then_see_payload(world: &mut WhisperWorld, expected: String) {
    assert_eq!(
        world.retrieved_payload.as_deref(),
        Some(expected.as_bytes()),
    );
}

#[then("the managed secret should not be found")]
async fn then_managed_not_found(world: &mut WhisperWorld) {
    assert!(
        world.retrieved_payload.is_none(),
        "expected managed secret to not be found"
    );
}

#[then("the managed secret should no longer exist")]
async fn then_managed_deleted(world: &mut WhisperWorld) {
    let id = world.managed_secret_id.unwrap();
    let found = world.managed_secret_repo.get_by_id(&id).await.unwrap();
    assert!(found.is_none(), "expected managed secret to be deleted");
}

#[then("no error should occur")]
async fn then_no_error(world: &mut WhisperWorld) {
    assert!(
        world.last_error.is_none(),
        "unexpected error: {:?}",
        world.last_error
    );
}
