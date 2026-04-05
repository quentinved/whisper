Feature: Create a shared secret

  Scenario: Successfully create a secret
    Given a secret "my-password" with expiration in 1 hours and self-destruct enabled
    When I create the secret
    Then the secret should be stored successfully

  Scenario: Successfully create a secret without self-destruct
    Given a secret "my-password" with expiration in 1 hours and self-destruct disabled
    When I create the secret
    Then the secret should be stored successfully
    And self-destruct should be disabled

  Scenario: Reject a secret that is too large
    Given a secret larger than 64 KB with expiration in 1 hours
    When I try to create the secret
    Then I should get a "Secret too large" error
