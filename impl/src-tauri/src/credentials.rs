use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const SERVICE: &str = "ZCode Credential Encryption";
const SYSTEM_PREFIX: &str = "enc:v2:";

#[derive(Debug, Clone, Serialize)]
pub struct ZCodeStatus {
    pub connected: bool,
    pub expires_at: Option<String>,
    pub message: String,
}

pub fn credentials_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "无法确定用户主目录".to_string())?;
    Ok(PathBuf::from(home)
        .join(".zcode")
        .join("v2")
        .join("credentials.json"))
}

/// Reads and decrypts the same `zcodejwttoken` the ZCode desktop app uses.
/// The token never leaves this process except on requests to the ZCode ASR API.
pub fn load_token() -> Result<String, String> {
    if let Ok(token) = std::env::var("SAYLINE_ZCODE_TOKEN") {
        let token = token.trim();
        if !token.is_empty() {
            return Ok(token.to_string());
        }
    }

    let path = credentials_path()?;
    let raw = fs::read_to_string(&path).map_err(|_| {
        "未找到 ZCode 登录凭据。请先打开 ZCode 桌面端并完成登录。".to_string()
    })?;
    let store: serde_json::Value =
        serde_json::from_str(&raw).map_err(|_| "ZCode 凭据文件格式异常".to_string())?;
    let value = store
        .get("zcodejwttoken")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "ZCode 凭据中没有登录 token。请重新登录 ZCode。".to_string())?;
    decrypt_value(&path, value)
}

pub fn status() -> ZCodeStatus {
    match load_token() {
        Ok(token) => {
            let expires_at = jwt_expiry(&token).map(|seconds| {
                chrono::DateTime::from_timestamp(seconds, 0)
                    .map(|time| time.with_timezone(&chrono::Local).to_rfc3339())
                    .unwrap_or_else(|| seconds.to_string())
            });
            ZCodeStatus {
                connected: true,
                expires_at,
                message: "已连接 ZCode 登录态".to_string(),
            }
        }
        Err(message) => ZCodeStatus {
            connected: false,
            expires_at: None,
            message,
        },
    }
}

fn decrypt_value(file: &Path, value: &str) -> Result<String, String> {
    if let Some(rest) = value.strip_prefix(SYSTEM_PREFIX) {
        let (source, rest) = rest
            .split_once(':')
            .ok_or_else(|| "ZCode 凭据版本异常".to_string())?;
        let (fingerprint, payload) = rest
            .split_once(':')
            .ok_or_else(|| "ZCode 凭据版本异常".to_string())?;
        if fingerprint.len() != 24 {
            return Err("ZCode 凭据指纹异常".to_string());
        }
        let key = match source {
            "os" => system_key(file)?,
            "env" => managed_secret_key(file)
                .ok_or_else(|| "ZCode 使用文件密钥，但未找到 credential-secret".to_string())?,
            _ => return Err("不支持的 ZCode 凭据来源".to_string()),
        };
        let actual = hex_lower(&Sha256::digest(&key))[..24].to_string();
        if actual != fingerprint {
            return Err("ZCode 凭据密钥不匹配，请重启 ZCode 后重试".to_string());
        }
        return open_payload(payload, &key);
    }

    if value.starts_with("enc:") {
        return Err("不支持的 ZCode 凭据格式".to_string());
    }
    Ok(value.to_string())
}

fn open_payload(payload: &str, key: &[u8]) -> Result<String, String> {
    let parts: Vec<&str> = payload.split('.').collect();
    if parts.len() != 3 {
        return Err("ZCode 凭据密文格式异常".to_string());
    }
    let iv = URL_SAFE_NO_PAD
        .decode(parts[0])
        .map_err(|_| "ZCode 凭据 IV 异常".to_string())?;
    let tag = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|_| "ZCode 凭据认证标签异常".to_string())?;
    let ciphertext = URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|_| "ZCode 凭据密文异常".to_string())?;
    if iv.len() != 12 || tag.len() != 16 {
        return Err("ZCode 凭据密文长度异常".to_string());
    }
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| "ZCode 凭据密钥长度异常".to_string())?;
    let mut combined = ciphertext;
    combined.extend_from_slice(&tag);
    let plain = cipher
        .decrypt(Nonce::from_slice(&iv), combined.as_ref())
        .map_err(|_| "ZCode 凭据解密失败，请重启 ZCode 后重试".to_string())?;
    String::from_utf8(plain).map_err(|_| "ZCode 凭据内容不是 UTF-8".to_string())
}

fn system_key(file: &Path) -> Result<Vec<u8>, String> {
    let account = hex_lower(&Sha256::digest(canonical(file).as_os_str().as_encoded_bytes()));
    let entry = keyring::Entry::new(SERVICE, &account)
        .map_err(|_| "无法打开系统凭据库".to_string())?;
    let encoded = entry.get_password().map_err(|_| {
        "无法读取 ZCode 的 keyring 主密钥。请确认系统钥匙串已解锁。".to_string()
    })?;
    let key = URL_SAFE_NO_PAD
        .decode(encoded.trim())
        .map_err(|_| "ZCode keyring 主密钥格式异常".to_string())?;
    if key.len() != 32 {
        return Err("ZCode keyring 主密钥长度异常".to_string());
    }
    Ok(key)
}

fn managed_secret_key(file: &Path) -> Option<Vec<u8>> {
    if let Ok(secret) = std::env::var("ZCODE_CREDENTIAL_SECRET") {
        if !secret.is_empty() {
            return Some(Sha256::digest(secret.as_bytes()).to_vec());
        }
    }
    let path = file.parent()?.join("credential-secret");
    let secret = fs::read_to_string(path).ok()?;
    if secret.len() < 32 || secret.len() > 4096 {
        return None;
    }
    Some(Sha256::digest(secret.as_bytes()).to_vec())
}

fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn jwt_expiry(token: &str) -> Option<i64> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    value.get("exp").and_then(|value| value.as_i64())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_is_lowercase_and_padded() {
        assert_eq!(hex_lower(&[0, 0xab, 0xff]), "00abff");
    }

    #[test]
    fn legacy_credentials_are_rejected() {
        let error = decrypt_value(Path::new("/nonexistent"), "enc:v1:abc").unwrap_err();
        assert!(error.contains("不支持"));
    }
}
