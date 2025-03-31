//! Contains JWT utils.
//!
//! # Example
//!
//! ```rust,no_run
//! use caslex_extra::security::jwt::{decode_token, encode_token, expiry};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Claims {
//!     sub: String,
//!     exp: u64,
//! }
//!
//! // generate expiry 60 seconds
//! let exp = expiry(60);
//!
//! let claims = Claims {
//!     sub: "123".to_owned(),
//!     exp,
//! };
//!
//! let encoded_token = encode_token(&claims).unwrap();
//! let decoded_token = decode_token::<Claims>(&encoded_token).unwrap();
//!
//! assert_eq!(decoded_token.claims.sub, claims.sub);
//! ```
//!
//! JWT secret key configure via environment variable `JWT_SECRET`.

use std::sync::LazyLock;

use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode, errors::Error,
    get_current_timestamp,
};
use serde::{Serialize, de::DeserializeOwned};

static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    let secret = std::env::var("JWT_SECRET")
        .expect("environment variable must be set for using jwt: JWT_SECRET");
    Keys::new(secret.as_bytes())
});

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

/// Returns expiry seconds.
pub fn expiry(secs_valid_for: u64) -> u64 {
    get_current_timestamp() + secs_valid_for
}

/// Encode token.
pub fn encode_token<T: Serialize>(claims: &T) -> Result<String, Error> {
    encode(&Header::default(), &claims, &KEYS.encoding)
}

/// Decode token.
pub fn decode_token<T: DeserializeOwned>(token: &str) -> Result<TokenData<T>, Error> {
    decode::<T>(token, &KEYS.decoding, &Validation::default())
}
