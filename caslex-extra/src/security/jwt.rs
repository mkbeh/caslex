use std::sync::LazyLock;

use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode, errors::Error,
    get_current_timestamp,
};
use serde::{Serialize, de::DeserializeOwned};

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET env must be set");
    Keys::new(secret.as_bytes())
});

pub fn expiry(secs_valid_for: u64) -> u64 {
    get_current_timestamp() + secs_valid_for
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub fn encode_token<T: Serialize>(claims: &T) -> Result<String, Error> {
    encode(&Header::default(), &claims, &KEYS.encoding)
}

pub fn decode_token<T: DeserializeOwned>(token: &str) -> Result<TokenData<T>, Error> {
    decode::<T>(token, &KEYS.decoding, &Validation::default())
}
