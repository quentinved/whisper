// swift-tools-version: 6.0
// Hexagonal layout mirroring the Rust workspace:
//   WhisperCore   ≙ services-core                 (domain, ports, commands)
//   WhisperCrypto ≙ adapters/aes-gcm-crypto       (SecretEncryption via CryptoKit)
//   WhisperHTTP   ≙ adapters/postgresql-adapter   (SharedSecretRepository over the Whisper API)
// The iOS app target is the application layer (≙ applications/axum/server).
import PackageDescription

let package = Package(
    name: "WhisperKit",
    platforms: [.iOS(.v17), .macOS(.v14)],
    products: [
        .library(name: "WhisperCore", targets: ["WhisperCore"]),
        .library(name: "WhisperCrypto", targets: ["WhisperCrypto"]),
        .library(name: "WhisperHTTP", targets: ["WhisperHTTP"]),
        // macOS harness for Tools/interop_check.sh and Tools/e2e_check.sh.
        .executable(name: "whisper-probe", targets: ["WhisperProbe"]),
    ],
    targets: [
        .target(name: "WhisperCore"),
        .target(name: "WhisperCrypto", dependencies: ["WhisperCore"]),
        .target(name: "WhisperHTTP", dependencies: ["WhisperCore"]),
        .executableTarget(
            name: "WhisperProbe",
            dependencies: ["WhisperCore", "WhisperCrypto", "WhisperHTTP"]
        ),
        .testTarget(name: "WhisperCoreTests", dependencies: ["WhisperCore"]),
        .testTarget(name: "WhisperCryptoTests", dependencies: ["WhisperCrypto"]),
        .testTarget(name: "WhisperHTTPTests", dependencies: ["WhisperHTTP"]),
    ]
)
