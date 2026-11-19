//! Cross-language interop harness for the ephemeral (zero-knowledge) payload
//! format shared by the web UI, CLI, Slack, Discord and the iOS app.
//!
//! The format is `nonce[12] ‖ ciphertext ‖ tag[16]`, base64url-no-pad — which is
//! byte-for-byte what Apple's CryptoKit calls `AES.GCM.SealedBox.combined`.
//! `applications/ios/Tools/interop_check.sh` runs this against the Swift
//! implementation in both directions.
//!
//!   cargo run -p aes-gcm-crypto --example interop_vector -- encrypt "plaintext"
//!   cargo run -p aes-gcm-crypto --example interop_vector -- decrypt <key_b64url> <payload_b64url>

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("encrypt") => {
            let plaintext = args.get(2).map(String::as_str).unwrap_or("interop");
            let (key, payload) =
                aes_gcm_crypto::encrypt_ephemeral(plaintext).expect("encryption failed");
            println!("{} {}", key, base64_url::encode(&payload));
            ExitCode::SUCCESS
        }
        Some("decrypt") => {
            let key = args.get(2).expect("usage: decrypt <key> <payload>");
            let payload = base64_url::decode(args.get(3).expect("usage: decrypt <key> <payload>"))
                .expect("payload is not base64url");
            match aes_gcm_crypto::decrypt_ephemeral(key, &payload) {
                Ok(plaintext) => {
                    println!("{}", plaintext);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("decrypt failed: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        _ => {
            eprintln!("usage: interop_vector <encrypt|decrypt> ...");
            ExitCode::FAILURE
        }
    }
}
