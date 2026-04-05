Feature: Delete expired secrets

  Scenario: Expired secrets are cleaned up
    Given a stored secret that has expired
    And a stored secret that has not expired
    When I delete expired secrets
    Then 1 expired secret should be deleted
    And 1 secret should remain

  Scenario: No expired secrets to delete
    Given a stored secret that has not expired
    When I delete expired secrets
    Then 0 expired secrets should be deleted
    And 1 secret should remain
