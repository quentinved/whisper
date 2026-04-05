Feature: Upsert a managed secret

  Scenario: Insert a new managed secret
    Given a managed secret payload "api-key-123"
    When I upsert the managed secret
    Then the upsert should indicate a new insert

  Scenario: Update an existing managed secret
    Given an existing managed secret with payload "old-value"
    When I upsert the managed secret with payload "new-value"
    Then the upsert should indicate an update

  Scenario: Reject an empty payload
    Given an empty managed secret payload
    When I try to upsert the managed secret
    Then I should get a "Payload is empty" error
