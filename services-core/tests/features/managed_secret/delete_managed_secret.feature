Feature: Delete a managed secret

  Scenario: Successfully delete a managed secret
    Given an existing managed secret with payload "to-delete"
    When I delete the managed secret
    Then the managed secret should no longer exist

  Scenario: Deleting a non-existent managed secret is idempotent
    Given a random secret ID for managed secrets
    When I delete the managed secret
    Then no error should occur
