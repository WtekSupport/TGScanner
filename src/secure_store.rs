use anyhow::{anyhow, Context, Result};
use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use rand::RngCore;
use std::fs;
use std::path::{Path, PathBuf};

const MAGIC: &[u8; 4] = b"TGUC";
const FORMAT_VERSION: u8 = 1;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

const KEYRING_SERVICE: &str = "TGScanner";
const KEYRING_USER: &str = "WtekSupport-vault-key";
const KEY_FILE: &str = ".tguser_vault.key";

pub fn load_plain_or_decrypt(path: &Path) -> Result<Vec<u8>> {
    let data = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    if data.starts_with(MAGIC) {
        decrypt_blob(&data, path)
    } else {
        Ok(data)
    }
}

pub fn save_encrypted(path: &Path, plaintext: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let blob = encrypt_blob(plaintext, path)?;
    fs::write(path, blob).with_context(|| format!("write {}", path.display()))?;
    restrict_permissions(path);
    Ok(())
}

pub fn migrate_legacy_file(legacy: &Path, encrypted: &Path) -> Result<bool> {
    if encrypted.is_file() || !legacy.is_file() {
        return Ok(false);
    }
    let plain = fs::read(legacy).context("read legacy config")?;
    save_encrypted(encrypted, &plain)?;
    let backup = legacy.with_extension("json.bak");
    let _ = fs::rename(legacy, backup);
    Ok(true)
}

fn vault_base(vault_path: &Path) -> Result<PathBuf> {
    vault_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow!("vault path has no parent directory"))
}

fn key_file_path(base: &Path) -> PathBuf {
    base.join(KEY_FILE)
}

fn read_key_file(path: &Path) -> Result<[u8; KEY_LEN]> {
    let data = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    if data.len() != KEY_LEN {
        return Err(anyhow!("invalid key file length"));
    }
    let mut key = [0u8; KEY_LEN];
    key.copy_from_slice(&data);
    Ok(key)
}

/// Stable random key stored next to the executable (same folder as vault files).
fn ensure_file_master_key(base: &Path) -> Result<[u8; KEY_LEN]> {
    let path = key_file_path(base);
    if path.is_file() {
        return read_key_file(&path);
    }
    let key = random_key();
    fs::write(&path, key).context("write vault key file")?;
    restrict_permissions(&path);
    Ok(key)
}

fn persist_file_master_key(base: &Path, key: &[u8; KEY_LEN]) -> Result<()> {
    let path = key_file_path(base);
    fs::write(&path, key).context("write vault key file")?;
    restrict_permissions(&path);
    Ok(())
}

fn candidate_decrypt_keys(base: &Path) -> Vec<[u8; KEY_LEN]> {
    let mut keys = Vec::new();
    let mut push = |k: [u8; KEY_LEN]| {
        if !keys.iter().any(|x| x == &k) {
            keys.push(k);
        }
    };

    if let Ok(k) = read_key_file(&key_file_path(base)) {
        push(k);
    }
    if let Ok(k) = keyring_master_key() {
        push(k);
    }
    if let Ok(k) = machine_fallback_key() {
        push(k);
    }
    keys
}

fn encrypt_blob(plaintext: &[u8], vault_path: &Path) -> Result<Vec<u8>> {
    let base = vault_base(vault_path)?;
    let key = ensure_file_master_key(&base)?;
    encrypt_with_key(plaintext, &key)
}

fn encrypt_with_key(plaintext: &[u8], key: &[u8; KEY_LEN]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new_from_slice(key).map_err(|e| anyhow!("{e}"))?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow!("encrypt: {e}"))?;
    let mut out = Vec::with_capacity(4 + 1 + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.push(FORMAT_VERSION);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

fn decrypt_blob(data: &[u8], vault_path: &Path) -> Result<Vec<u8>> {
    if data.len() < 4 + 1 + NONCE_LEN + 16 {
        return Err(anyhow!("encrypted blob too short"));
    }
    if &data[..4] != MAGIC {
        return Err(anyhow!("invalid magic"));
    }
    if data[4] != FORMAT_VERSION {
        return Err(anyhow!("unsupported vault version"));
    }

    let base = vault_base(vault_path)?;
    let keys = candidate_decrypt_keys(&base);
    if keys.is_empty() {
        return Err(anyhow!("no vault keys available"));
    }

    let mut last_err = None;
    for key in keys {
        match decrypt_with_key(data, &key) {
            Ok(plaintext) => {
                let _ = persist_file_master_key(&base, &key);
                return Ok(plaintext);
            }
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow!("decrypt failed")))
}

fn decrypt_with_key(data: &[u8], key: &[u8; KEY_LEN]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new_from_slice(key).map_err(|e| anyhow!("{e}"))?;
    let nonce = Nonce::from_slice(&data[5..5 + NONCE_LEN]);
    cipher
        .decrypt(nonce, &data[5 + NONCE_LEN..])
        .map_err(|e| anyhow!("decrypt: {e}"))
}

fn keyring_master_key() -> Result<[u8; KEY_LEN]> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)?;
    match entry.get_password() {
        Ok(stored) => decode_key_hex(&stored),
        Err(keyring::Error::NoEntry) => Err(anyhow!("keyring empty")),
        Err(e) => Err(anyhow!("keyring: {e}")),
    }
}

fn machine_fallback_key() -> Result<[u8; KEY_LEN]> {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b"tguser-v1-local-vault");
    if let Ok(u) = std::env::var("USERNAME").or_else(|_| std::env::var("USER")) {
        h.update(u.as_bytes());
    }
    if let Ok(hn) = std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")) {
        h.update(hn.as_bytes());
    }
    if let Ok(exe) = std::env::current_exe() {
        h.update(exe.to_string_lossy().as_bytes());
    }
    let digest = h.finalize();
    let mut key = [0u8; KEY_LEN];
    key.copy_from_slice(&digest[..KEY_LEN]);
    Ok(key)
}

fn random_key() -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    OsRng.fill_bytes(&mut key);
    key
}

fn decode_key_hex(s: &str) -> Result<[u8; KEY_LEN]> {
    let s = s.trim();
    if s.len() != KEY_LEN * 2 {
        return Err(anyhow!("invalid key length in keyring"));
    }
    let mut key = [0u8; KEY_LEN];
    for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
        if i >= KEY_LEN {
            break;
        }
        let hex = std::str::from_utf8(chunk).context("key hex utf8")?;
        key[i] = u8::from_str_radix(hex, 16).context("key hex parse")?;
    }
    Ok(key)
}

fn restrict_permissions(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    let _ = path;
}

pub fn session_path(base: &Path) -> PathBuf {
    base.join(".tguser_session.enc")
}

pub fn legacy_session_path(base: &Path) -> PathBuf {
    base.join(".tguser_session")
}
