use anyhow::{anyhow, Result};
use indexed_db_futures::{database::Database, prelude::*, transaction::TransactionMode};
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::JsValue;

#[derive(Serialize, Deserialize, Clone)]
pub struct SentMessage {
    pub sender_id: i64,
    pub receiver_id: i64,
    pub message: String,
    pub timestamp: i64,
}

pub enum KeyType {
    Public,
    Private,
    UserId,
}

impl KeyType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
            Self::UserId => "user_id",
        }
    }
}

pub enum StoredValue {
    Bytes(Vec<u8>),
    UserId(i64),
}

async fn open_db() -> Result<Database> {
    Database::open("e2ee")
        .with_version(1u8)
        .with_on_upgrade_needed(|_, db| {
            db.create_object_store("keys").build()?;
            db.create_object_store("messages")
                .with_auto_increment(true)
                .build()?;

            Ok(())
        })
        .await
        .map_err(|e| anyhow!("Failed to open IndexedDB: {e}"))
}

pub async fn save(key: KeyType, value: StoredValue) -> Result<()> {
    let db = open_db().await?;

    let tx = db
        .transaction("keys")
        .with_mode(TransactionMode::Readwrite)
        .build()
        .map_err(|e| anyhow!("Failed to create transaction: {e}"))?;

    let store = tx
        .object_store("keys")
        .map_err(|e| anyhow!("Failed to open object store: {e}"))?;

    let value = match value {
        StoredValue::Bytes(data) => JsValue::from(Uint8Array::from(data.as_slice())),
        StoredValue::UserId(id) => JsValue::from_f64(id as f64),
    };

    store
        .put(value)
        .with_key(JsValue::from_str(key.as_str()))
        .await
        .map_err(|e| anyhow!("Failed to save value: {e}"))?;

    tx.commit()
        .await
        .map_err(|e| anyhow!("Failed to commit transaction: {e}"))?;

    Ok(())
}

pub async fn get(key: KeyType) -> Result<StoredValue> {
    let db = open_db().await?;

    let tx = db
        .transaction("keys")
        .with_mode(TransactionMode::Readonly)
        .build()
        .map_err(|e| anyhow!("Failed to create transaction: {e}"))?;

    let store = tx
        .object_store("keys")
        .map_err(|e| anyhow!("Failed to open object store: {e}"))?;

    let value = store
        .get(JsValue::from_str(key.as_str()))
        .await
        .map_err(|e| anyhow!("Failed to get value: {e}"))?
        .ok_or_else(|| anyhow!("Key not found"))?;

    tx.commit()
        .await
        .map_err(|e| anyhow!("Failed to commit transaction: {e}"))?;

    match key {
        KeyType::Public | KeyType::Private => {
            Ok(StoredValue::Bytes(Uint8Array::new(&value).to_vec()))
        }
        KeyType::UserId => {
            let id = value
                .as_f64()
                .ok_or_else(|| anyhow!("Stored value is not a number"))?;

            Ok(StoredValue::UserId(id as i64))
        }
    }
}

pub async fn save_message(message: SentMessage) -> Result<()> {
    let db = open_db().await?;

    let tx = db
        .transaction("messages")
        .with_mode(TransactionMode::Readwrite)
        .build()
        .map_err(|e| anyhow!("Failed to create transaction: {e}"))?;

    let store = tx
        .object_store("messages")
        .map_err(|e| anyhow!("Failed to open object store: {e}"))?;

    store
        .add(to_value(&message).map_err(|e| anyhow!("Failed to serialize message: {e}"))?)
        .await
        .map_err(|e| anyhow!("Failed to save message: {e}"))?;

    tx.commit()
        .await
        .map_err(|e| anyhow!("Failed to commit transaction: {e}"))?;

    Ok(())
}

pub async fn get_messages() -> Result<Vec<SentMessage>> {
    let db = open_db().await?;

    let tx = db
        .transaction("messages")
        .with_mode(TransactionMode::Readonly)
        .build()
        .map_err(|e| anyhow!("Failed to create transaction: {e}"))?;

    let store = tx
        .object_store("messages")
        .map_err(|e| anyhow!("Failed to open object store: {e}"))?;

    let values = store
        .get_all()
        .await
        .map_err(|e| anyhow!("Failed to retrieve messages: {e}"))?;

    tx.commit()
        .await
        .map_err(|e| anyhow!("Failed to commit transaction: {e}"))?;

    values
        .into_iter()
        .map(|value| {
            from_value(value.expect("Something went wrong!"))
                .map_err(|e| anyhow!("Failed to deserialize message: {e}"))
        })
        .collect()
}
