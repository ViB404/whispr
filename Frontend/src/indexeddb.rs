use anyhow::{anyhow, Result};
use indexed_db_futures::{database::Database, prelude::*, transaction::TransactionMode};
use js_sys::Uint8Array;
use wasm_bindgen::JsValue;

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
