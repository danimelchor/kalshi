use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use rand::rngs::OsRng;
use rsa::Pss;
use rsa::traits::SignatureScheme;
use rsa::{RsaPrivateKey, pkcs1::DecodeRsaPrivateKey};
use sha2::{Digest, Sha256};
use std::fmt::Display;
use std::{env, path::PathBuf};

use tokio::fs;

#[derive(Clone)]
pub struct ApiKey(String);

impl ApiKey {
    pub fn from_env() -> Result<Self> {
        let key = env::var("KALSHI_API_KEY").context("Kalshi API key is not set")?;
        Ok(Self(key))
    }
}

impl Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone)]
pub struct PrivateKey(RsaPrivateKey);

impl PrivateKey {
    pub async fn from_file(path: PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        let private_key =
            RsaPrivateKey::from_pkcs1_pem(&content).context("Error decoding private RSA key")?;
        Ok(Self(private_key))
    }

    pub fn sign(&self, msg: &str) -> Result<String> {
        let padding = Pss::new_with_salt::<Sha256>(32);

        let mut hasher = Sha256::new();
        hasher.update(msg.as_bytes());
        let digest = hasher.finalize();

        let signature = padding
            .sign(Some(&mut OsRng), &self.0, &digest)
            .context("signing message")?;
        Ok(general_purpose::STANDARD.encode(signature))
    }
}
