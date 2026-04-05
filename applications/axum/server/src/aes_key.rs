use std::path::Path;
use std::{fs::File, io::Read};

pub fn load_aes_key(key_path: &str) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let path = Path::new(key_path);

    // Check file permissions on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            let mode = metadata.mode();
            if mode & 0o077 != 0 {
                tracing::warn!(
                    "AES key file '{}' has overly permissive permissions ({:o}). \
                     Consider restricting with: chmod 600 {}",
                    key_path,
                    mode & 0o777,
                    key_path
                );
            }
        }
    }

    let mut key_data = [0u8; 32];
    let mut file = File::open(path).map_err(|e| {
        format!(
            "Failed to open AES key file '{}': {}. \
             Please generate it with: openssl rand -out {} 32",
            key_path, e, key_path
        )
    })?;

    file.read_exact(&mut key_data).map_err(|e| {
        format!(
            "Failed to read AES key from '{}': {}. \
             The file must be exactly 32 bytes. \
             Regenerate it with: openssl rand -out {} 32",
            key_path, e, key_path
        )
    })?;

    let mut extra = [0u8; 1];
    if file.read(&mut extra).is_ok_and(|n| n > 0) {
        return Err(format!(
            "AES key file '{}' is longer than 32 bytes. \
             The file must be exactly 32 bytes. \
             Regenerate it with: openssl rand -out {} 32",
            key_path, key_path
        )
        .into());
    }

    Ok(key_data)
}
