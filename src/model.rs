use serde_aux::prelude::*;
use std::fmt;

pub const API_ENDPOINT: &str = "https://discord.com/api/v8";
pub const HEX_BASE: [&str; 16] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f",
];

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, sqlx::Type)]
pub enum Verification {
    None,
    Yanked,
    Unsafe,
    Auto,
    Manual,
    Core,
}

impl Default for Verification {
    fn default() -> Self {
        Self::None
    }
}

impl fmt::Display for Verification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Yanked => write!(f, "Yanked"),
            Self::Unsafe => write!(f, "Unsafe"),
            Self::Auto => write!(f, "Auto"),
            Self::Manual => write!(f, "Manual"),
            Self::Core => write!(f, "Core"),
        }
    }
}

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
    pub email: String,
}

fn default_address() -> String {
    "127.0.0.1".to_string()
}
fn default_port() -> u16 {
    8000
}
fn default_workers() -> usize {
    1
}
fn default_keep_alive() -> usize {
    30
}
fn default_log() -> String {
    "indo,sqlx=warn".to_string()
}
fn default_redis() -> String {
    "127.0.0.1:6379".to_string()
}
fn default_path() -> String {
    "./tmp".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    // Web Server
    #[serde(default = "default_address")]
    pub address: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_workers")]
    pub workers: usize,
    #[serde(default = "default_keep_alive")]
    pub keep_alive: usize,
    #[serde(default = "default_log")]
    pub log: String,

    // Private Cookies
    pub secret_key: String,
    pub iv_key: String,

    // Discord
    pub oauth2_url: String,
    pub client_id: u64,
    pub client_secret: String,
    pub redirect_uri: String,

    // Other
    #[serde(default = "default_redis")]
    pub redis_uri: String,
    #[serde(default = "default_path")]
    pub mods_path: String,
}

bitflags! {
    pub struct Roles: u32 {
        const OWNER    = 0b00000001;
        const ADMIN    = 0b00000010;
        const MOD      = 0b00000100;
        const VERIFYER = 0b00001000;
        const MAPPER   = 0b00010000;
        const BOT      = 0b00100000;
    }
}

impl Default for Roles {
    fn default() -> Roles {
        Roles::from_bits_truncate(0)
    }
}

bitflags! {
    pub struct TeamRoles: u32 {
        const OWNER    = 0b00000001;
        const ADMIN    = 0b00000010;
        const MOD      = 0b00000100;
    }
}

impl Default for TeamRoles {
    fn default() -> TeamRoles {
        TeamRoles::from_bits_truncate(0)
    }
}
