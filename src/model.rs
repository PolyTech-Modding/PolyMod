use serde_aux::prelude::*;

pub const API_ENDPOINT: &str = "https://discord.com/api/v8";

#[derive(Serialize, Deserialize, Debug)]
pub struct OAuthTokenData {
    pub client_id: u64,
    pub client_secret: String,
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
    pub scope: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OAuthResponse {
    pub access_token: String,
    pub expires_in: u32,
    pub refresh_token: String,
    pub scope: String,
    pub token_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserResponse {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: u64,
    pub username: String,
    pub avatar: Option<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub discriminator: u16,
    pub public_flags: u64,
    pub flags: u64,
    pub locale: String,
    pub mfa_enabled: bool,
    pub premium_type: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    // Web Server
    pub address: String,
    pub port: u16,
    pub workers: usize,
    pub keep_alive: usize,
    pub log: String,

    // Private Cookies
    pub secret_key: String,

    // Discord
    pub oauth2_url: String,
    pub client_id: u64,
    pub client_secret: String,
    pub redirect_uri: String,
}
