Feature: Retrieve a managed secret

  Scenario: Successfully retrieve a managed secret
    Given an existing managed secret with payload "my-api-key"
    When I retrieve the managed secret by its ID
    Then I should see the managed payload "my-api-key"

  Scenario: Retrieve a non-existent managed secret
    Given a random secret ID for managed secrets
    When I retrieve the managed secret by its ID
    Then the managed secret should not be found
