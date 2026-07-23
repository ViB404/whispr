use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct RegisterRequest {
    pub username: String,
    pub public_key: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterResponse {
    pub id: i64,
    pub message: String,
}

pub async fn register_user(username: String, public_key: Vec<u8>) -> Result<RegisterResponse> {
    let client = reqwest::Client::new();

    let response = client
        .post("http://127.0.0.1:6767/register")
        .json(&RegisterRequest {
            username,
            public_key,
        })
        .send()
        .await?;

    if !response.status().is_success() {
        bail!("{}", response.text().await?);
    }

    Ok(response.json().await?)
}
