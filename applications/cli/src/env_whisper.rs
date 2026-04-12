use crate::error::CliError;
use std::collections::BTreeMap;
use std::path::Path;

pub const ENV_WHISPER_FILE: &str = ".env.whisper";
const ENV_FILE: &str = ".env";

/// Read the local .env file into a BTreeMap of key=value pairs.
/// Returns an empty map if .env doesn't exist. Warns on permission errors.
pub fn read_env_file() -> BTreeMap<String, String> {
    let content = match std::fs::read_to_string(ENV_FILE) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return BTreeMap::new(),
        Err(e) => {
            eprintln!("warning: could not read {}: {}", ENV_FILE, e);
            return BTreeMap::new();
        }
    };
    content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn read_from(path: &Path) -> Result<BTreeMap<String, String>, CliError> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }

    let content = std::fs::read_to_string(path).map_err(CliError::EnvWhisperRead)?;

    let mut entries = BTreeMap::new();
    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, uuid) = line
            .split_once('=')
            .ok_or_else(|| CliError::EnvWhisperMalformed {
                line: line_num + 1,
                content: line.to_string(),
            })?;
        entries.insert(name.trim().to_string(), uuid.trim().to_string());
    }

    Ok(entries)
}

fn write_to(path: &Path, entries: &BTreeMap<String, String>) -> Result<(), CliError> {
    let content: String = entries
        .iter()
        .map(|(name, uuid)| format!("{}={}", name, uuid))
        .collect::<Vec<String>>()
        .join("\n");
    std::fs::write(path, content + "\n").map_err(CliError::EnvWhisperWrite)?;
    Ok(())
}

pub fn read() -> Result<BTreeMap<String, String>, CliError> {
    read_from(Path::new(ENV_WHISPER_FILE))
}

pub fn write(entries: &BTreeMap<String, String>) -> Result<(), CliError> {
    write_to(Path::new(ENV_WHISPER_FILE), entries)
}

pub fn set(name: &str, uuid: &str) -> Result<(), CliError> {
    let mut entries = read()?;
    entries.insert(name.to_string(), uuid.to_string());
    write(&entries)
}

pub fn remove(name: &str) -> Result<Option<String>, CliError> {
    let mut entries = read()?;
    let uuid = entries.remove(name);
    write(&entries)?;
    Ok(uuid)
}

pub fn get(name: &str) -> Result<Option<String>, CliError> {
    Ok(read()?.get(name).cloned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn env_whisper_path(dir: &TempDir) -> std::path::PathBuf {
        dir.path().join(".env.whisper")
    }

    #[test]
    fn test_read_nonexistent_file() {
        let dir = TempDir::new().unwrap();
        let path = env_whisper_path(&dir);
        let entries = read_from(&path).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_set_and_get() {
        let dir = TempDir::new().unwrap();
        let path = env_whisper_path(&dir);
        let mut entries = BTreeMap::new();
        entries.insert(
            "DB_PASSWORD".to_string(),
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
        );
        write_to(&path, &entries).unwrap();
        let read_back = read_from(&path).unwrap();
        assert_eq!(
            read_back.get("DB_PASSWORD"),
            Some(&"550e8400-e29b-41d4-a716-446655440000".to_string())
        );
    }

    #[test]
    fn test_remove_entry() {
        let dir = TempDir::new().unwrap();
        let path = env_whisper_path(&dir);
        let mut entries = BTreeMap::new();
        entries.insert(
            "API_KEY".to_string(),
            "6ba7b810-9dad-11d1-80b4-00c04fd430c8".to_string(),
        );
        write_to(&path, &entries).unwrap();

        let mut entries = read_from(&path).unwrap();
        let removed = entries.remove("API_KEY");
        write_to(&path, &entries).unwrap();

        assert_eq!(
            removed,
            Some("6ba7b810-9dad-11d1-80b4-00c04fd430c8".to_string())
        );
        let final_entries = read_from(&path).unwrap();
        assert_eq!(final_entries.get("API_KEY"), None);
    }

    #[test]
    fn test_roundtrip_multiple_entries() {
        let dir = TempDir::new().unwrap();
        let path = env_whisper_path(&dir);
        let mut entries = BTreeMap::new();
        entries.insert("A".to_string(), "uuid-a".to_string());
        entries.insert("B".to_string(), "uuid-b".to_string());
        entries.insert("C".to_string(), "uuid-c".to_string());
        write_to(&path, &entries).unwrap();

        let read_back = read_from(&path).unwrap();
        assert_eq!(read_back.len(), 3);
        assert_eq!(read_back["A"], "uuid-a");
        assert_eq!(read_back["B"], "uuid-b");
        assert_eq!(read_back["C"], "uuid-c");
    }

    #[test]
    fn test_malformed_line_errors() {
        let dir = TempDir::new().unwrap();
        let path = env_whisper_path(&dir);
        fs::write(&path, "GOOD=uuid\nBADLINE\n").unwrap();
        let result = read_from(&path);
        assert!(result.is_err());
    }
}
