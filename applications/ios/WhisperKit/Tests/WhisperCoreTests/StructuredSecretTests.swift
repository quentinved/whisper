import Foundation
import Testing
import WhisperCore

@Suite struct StructuredSecretTests {
    typealias Row = StructuredSecret.Row
    typealias Section = StructuredSecret.Section

    // MARK: Reading what the web sends

    @Test func readsTheWebFormsOutputInTheSendersOrder() throws {
        // Exactly what `JSON.stringify` produces in create_secret.js.
        let web = #"{"DB_USER":"admin","DB_PASSWORD":"p@ss \"word\"","stripe":{"key":"sk_test_1","webhook":"whsec_2"}}"#

        let secret = try #require(StructuredSecret(parsing: web))

        #expect(
            secret.entries == [
                .value(key: "DB_USER", value: "admin"),
                .value(key: "DB_PASSWORD", value: "p@ss \"word\""),
                .section(
                    name: "stripe",
                    entries: [.value(key: "key", value: "sk_test_1"), .value(key: "webhook", value: "whsec_2")]),
            ])
    }

    @Test func readsHandWrittenJSONWithUnicodeLiteralsAndArrays() throws {
        let handWritten = """
              {
                "zeta": "first",
                "emoji": "\\ud83d\\udd10 caf\\u00e9",
                "port": 5432,
                "debug": false,
                "hosts": ["a", "b, c"],
                "nothing": null,
                "empty": {}
              }
            """

        let secret = try #require(StructuredSecret(parsing: handWritten))

        #expect(
            secret.entries == [
                .value(key: "zeta", value: "first"),
                .value(key: "emoji", value: "🔐 café"),
                .literal(key: "port", json: "5432"),
                .literal(key: "debug", json: "false"),
                .literal(key: "hosts", json: #"["a", "b, c"]"#),
                .literal(key: "nothing", json: "null"),
                .section(name: "empty", entries: []),
            ])
    }

    @Test(arguments: [
        "plain password",
        "{}",  // empty object: the web shows it as text
        "[1, 2]",
        "{not json}",
        #"{"a": "b"} trailing"#,
        "\"just a string\"",
    ])
    func treatsEverythingElseAsPlainText(text: String) {
        #expect(StructuredSecret(parsing: text) == nil)
    }

    // MARK: Building like the web form

    @Test func buildsCompactJSONTheWebCanRead() throws {
        let secret = try StructuredSecret(
            rows: [Row(key: " API_KEY ", value: "abc/123"), Row(key: "", value: "ignored"), Row(key: "skip", value: "")],
            sections: [
                Section(name: "db", rows: [Row(key: "user", value: "root"), Row(key: "pass", value: "x\ny")]),
                Section(name: "", rows: [Row(key: "", value: "")]),  // empty and unnamed: dropped
            ])

        #expect(secret.json == #"{"API_KEY":"abc/123","db":{"user":"root","pass":"x\ny"}}"#)
        #expect(StructuredSecret(parsing: secret.json) == secret)
    }

    @Test func prettyPrintsLikeJSONStringifyWithTwoSpaces() throws {
        let secret = try #require(StructuredSecret(parsing: #"{"a":"1","s":{"b":"2"},"n":3}"#))
        #expect(
            secret.prettyJSON == """
                {
                  "a": "1",
                  "s": {
                    "b": "2"
                  },
                  "n": 3
                }
                """)
    }

    @Test func refusesWhatTheWebFormRefuses() {
        #expect(throws: StructuredSecretError.empty) {
            try StructuredSecret(rows: [Row(key: "k", value: "")], sections: [])
        }
        #expect(throws: StructuredSecretError.duplicateKey("k")) {
            try StructuredSecret(rows: [Row(key: "k", value: "1"), Row(key: "k ", value: "2")], sections: [])
        }
        #expect(throws: StructuredSecretError.unnamedSection) {
            try StructuredSecret(rows: [], sections: [Section(name: " ", rows: [Row(key: "k", value: "v")])])
        }
        #expect(throws: StructuredSecretError.duplicateName("k")) {
            try StructuredSecret(
                rows: [Row(key: "k", value: "v")], sections: [Section(name: "k", rows: [Row(key: "a", value: "b")])])
        }
        #expect(throws: StructuredSecretError.duplicateKeyInSection(section: "s", key: "a")) {
            try StructuredSecret(
                rows: [],
                sections: [Section(name: "s", rows: [Row(key: "a", value: "1"), Row(key: "a", value: "2")])])
        }
    }
}
