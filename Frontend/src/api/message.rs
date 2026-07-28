use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct SendMessageRequest {
    pub sender_id: i64,
    pub receiver_id: i64,
    pub ciphertext: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageResponse {
    pub id: i64,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Message {
    pub id: i64,
    pub sender_id: i64,
    pub receiver_id: i64,
    pub ciphertext: Vec<u8>,
    pub created_at: i64,
}

pub async fn send_message(
    sender_id: i64,
    receiver_id: i64,
    ciphertext: Vec<u8>,
) -> Result<SendMessageResponse> {
    let client = reqwest::Client::new();

    let response = client
        .post("http://127.0.0.1:6767/messages")
        .json(&SendMessageRequest {
            sender_id,
            receiver_id,
            ciphertext,
        })
        .send()
        .await?;

    if !response.status().is_success() {
        bail!("{}", response.text().await?);
    }

    Ok(response.json().await?)
}

pub async fn get_message(message_id: i64) -> Result<Message> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://127.0.0.1:6767/messages/{message_id}"))
        .send()
        .await?;

    if !response.status().is_success() {
        bail!("{}", response.text().await?);
    }

    Ok(response.json().await?)
}

pub async fn get_conversation(user1: i64, user2: i64) -> Result<Vec<Message>> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!(
            "http://127.0.0.1:6767/messages/{user1}/{user2}"
        ))
        .send()
        .await?;

    if !response.status().is_success() {
        bail!("{}", response.text().await?);
    }

    Ok(response.json().await?)
}
