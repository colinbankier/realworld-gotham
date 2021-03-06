use jsonwebtoken::{encode, Header};
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

// TODO: get the secret from config
pub const SECRET: &str = "notsosecret";

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Claims {
    sub: i32,
    exp: u64,
}

impl Claims {
    pub fn user_id(&self) -> i32 {
        self.sub
    }
}

pub fn encode_token(sub: i32) -> String {
    encode(&Header::default(), &claims_for(sub, 3600), SECRET.as_ref()).unwrap()
}

pub fn claims_for(user_id: i32, expire_in: u64) -> Claims {
    Claims {
        sub: user_id,
        exp: seconds_from_now(expire_in),
    }
}

fn seconds_from_now(secs: u64) -> u64 {
    let expiry_time =
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + Duration::from_secs(secs);
    expiry_time.as_secs()
}
