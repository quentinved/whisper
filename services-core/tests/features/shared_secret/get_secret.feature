Feature: Retrieve a shared secret

  Scenario: Successfully retrieve a secret
    Given a stored secret "my-password" with self-destruct disabled
    When I retrieve the secret by its ID
    Then I should see the decrypted value "my-password"

  Scenario: Retrieve a non-existent secret
    Given a random secret ID
    When I retrieve the secret by its ID
    Then the secret should not be found

  Scenario: Retrieve a self-destructing secret
    Given a stored secret "one-time-password" with self-destruct enabled
    When I retrieve the secret by its ID
    Then I should see the decrypted value "one-time-password"
    And self-destruct should be enabled
