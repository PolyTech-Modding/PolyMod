use crate::utils::crypt::*;
use crypto::{digest::Digest, sha2::Sha256};
use std::time::SystemTime;

use base64::encode;
use std::convert::TryInto;

pub fn gen_token(user_id: u64, email: impl ToString, key: &[u8], iv: &[u8]) -> Option<String> {
    let key: [u8; 32] = key.try_into().ok()?;
    let iv: [u8; 16] = iv.try_into().ok()?;

    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?
        .as_micros();

    let mut hasher = Sha256::new();
    hasher.input_str(&format!(
        "{}:{}",
        email.to_string(),
        user_id as u128 ^ (time % user_id as u128)
    ));
    let initial_data = hasher.result_str();

    let encrypted_data = encrypt_bytes(initial_data.as_bytes(), key, iv).ok()?;
    let encrypted_data_text = encode(encrypted_data.to_vec());

    Some(encrypted_data_text)
}
