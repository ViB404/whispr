use anyhow::{anyhow, Result};
use indexed_db_futures::{database::Database, prelude::*, transaction::TransactionMode};
use wasm_bindgen::JsValue;

pub enum KeyType {
    Public,
    Private,
}

impl KeyType {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
        }
    }
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

pub async fn save_key(key: KeyType, data: Vec<u8>) -> Result<()> {
    let db = open_db().await?;

    let tx = db
        .transaction("keys")
        .with_mode(TransactionMode::Readwrite)
        .build()
        .map_err(|e| anyhow!("Failed to create transaction: {e}"))?;

    let store = tx
        .object_store("keys")
        .map_err(|e| anyhow!("Failed to open object store: {e}"))?;

    store
        .put(JsValue::from(js_sys::Uint8Array::from(data.as_slice())))
        .with_key(JsValue::from_str(key.as_str()))
        .await
        .map_err(|e| anyhow!("Failed to save key: {e}"))?;

    tx.commit()
        .await
        .map_err(|e| anyhow!("Failed to commit transaction: {e}"))?;

    Ok(())
}
