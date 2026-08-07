use crate::api::message::Message;
use crate::indexeddb::SentMessage;
use js_sys::futures::JsFuture;
use js_sys::{Array, Reflect, Uint8Array};
use std::collections::HashMap;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CryptoKey, TextDecoder, TextEncoder};

pub async fn encrypt_message(message: &str, key: &[u8]) -> Result<Vec<u8>, JsValue> {
    let encoder = TextEncoder::new()?;
    let data = encoder.encode_with_input(message);
    let window = web_sys::window().unwrap();
    let crypto = window.crypto()?;
    let subtle = crypto.subtle();

    let algorithm = js_sys::Object::new();
    Reflect::set(&algorithm, &"name".into(), &"RSA-OAEP".into())?;

    let hash = js_sys::Object::new();
    Reflect::set(&hash, &"name".into(), &"SHA-256".into())?;

    Reflect::set(&algorithm, &"hash".into(), &hash)?;

    let key_data = Uint8Array::from(key);

    let promise = subtle.import_key_with_object(
        "spki",
        &key_data.buffer(),
        &algorithm,
        true,
        &Array::of1(&"encrypt".into()),
    )?;
    let crypto_key: CryptoKey = JsFuture::from(promise).await?.dyn_into()?;

    let promise = subtle.encrypt_with_object_and_u8_array(&algorithm, &crypto_key, &data)?;

    let encrypted = JsFuture::from(promise).await?;

    let buffer: js_sys::ArrayBuffer = encrypted.dyn_into()?;
    let bytes = Uint8Array::new(&buffer).to_vec();

    Ok(bytes)
}

pub async fn decrypt_message(encrypted: &[u8], key: &[u8]) -> Result<String, JsValue> {
    let window = web_sys::window().unwrap();
    let crypto = window.crypto()?;
    let subtle = crypto.subtle();

    let algorithm = js_sys::Object::new();

    Reflect::set(&algorithm, &"name".into(), &"RSA-OAEP".into())?;

    let hash = js_sys::Object::new();
    Reflect::set(&hash, &"name".into(), &"SHA-256".into())?;

    Reflect::set(&algorithm, &"hash".into(), &hash)?;

    let key_data = Uint8Array::from(key);

    let promise = subtle.import_key_with_object(
        "pkcs8",
        &key_data.buffer(),
        &algorithm,
        true,
        &Array::of1(&"decrypt".into()),
    )?;

    let crypto_key: CryptoKey = JsFuture::from(promise).await?.dyn_into()?;

    let promise = subtle.decrypt_with_object_and_u8_array(&algorithm, &crypto_key, encrypted)?;

    let decrypted = JsFuture::from(promise).await?;

    let buffer: js_sys::ArrayBuffer = decrypted.dyn_into()?;

    let bytes = Uint8Array::new(&buffer).to_vec();

    let decoder = TextDecoder::new()?;
    let text = decoder.decode_with_u8_array(&bytes)?;

    Ok(text)
}

pub fn merge_messages(
    current_user_id: i64,
    local: Vec<SentMessage>,
    server: Vec<Message>,
) -> Vec<SentMessage> {
    let mut local_map: HashMap<(i64, i64, i64), SentMessage> = local
        .into_iter()
        .map(|m| ((m.sender_id, m.receiver_id, m.timestamp), m))
        .collect();

    let mut merged: Vec<SentMessage> = Vec::with_capacity(server.len());

    for msg in server {
        let timestamp = msg.created_at * 1000;

        if msg.sender_id == current_user_id {
            if let Some(local_msg) = local_map.remove(&(msg.sender_id, msg.receiver_id, timestamp))
            {
                merged.push(local_msg);
            } else {
                merged.push(SentMessage {
                    sender_id: msg.sender_id,
                    receiver_id: msg.receiver_id,
                    message: "Can't be decoded".to_string(),
                    timestamp,
                });
            }
        } else {
            merged.push(SentMessage {
                sender_id: msg.sender_id,
                receiver_id: msg.receiver_id,
                message: "Can't be decoded".to_string(),
                timestamp,
            });
        }
    }

    merged.sort_by_key(|m| m.timestamp);
    merged
}
