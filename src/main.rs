use dioxus::prelude::*;
use indexed_db_futures::{database::Database, prelude::*, transaction::TransactionMode};
use js_sys::futures::JsFuture;
use js_sys::wasm_bindgen::JsValue;
use js_sys::{Array, Object, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, CryptoKey};

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        h1 { "Generate Keys 🔑" }
        button {
            onclick: move |_| {
                spawn_local(async {
                    generate_keys().await.expect("Something Went Wrong");
                });
            },
            "Generate"
        }
    }
}

async fn generate_keys() -> Result<(), JsValue> {
    let subtle = window().unwrap().crypto()?.subtle();

    let algorithm = Object::new();
    Reflect::set(&algorithm, &"name".into(), &"RSA-OAEP".into())?;
    Reflect::set(
        &algorithm,
        &"modulusLength".into(),
        &JsValue::from_f64(2048.0),
    )?;
    Reflect::set(
        &algorithm,
        &"publicExponent".into(),
        &js_sys::Uint8Array::from(&[1, 0, 1][..]),
    )?;
    Reflect::set(&algorithm, &"hash".into(), &"SHA-256".into())?;

    let usages = Array::new();
    usages.push(&"encrypt".into());
    usages.push(&"decrypt".into());

    let promise = subtle.generate_key_with_object(&algorithm, true, &usages)?;
    let key_pair = JsFuture::from(promise).await?;

    let public: CryptoKey = Reflect::get(&key_pair, &"publicKey".into())?.dyn_into()?;
    let private: CryptoKey = Reflect::get(&key_pair, &"privateKey".into())?.dyn_into()?;

    let public_buffer = JsFuture::from(subtle.export_key("spki", &public)?).await?;
    let private_buffer = JsFuture::from(subtle.export_key("pkcs8", &private)?).await?;

    let public_bytes = js_sys::Uint8Array::new(&public_buffer).to_vec();
    let private_bytes = js_sys::Uint8Array::new(&private_buffer).to_vec();

    save_key(KeyType::Public, public_bytes).await?;
    save_key(KeyType::Private, private_bytes).await?;

    Ok(())
}

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
async fn open_db() -> Result<Database, JsValue> {
    let db = Database::open("e2ee")
        .with_version(1u8)
        .with_on_upgrade_needed(|_evt, db| {
            db.create_object_store("keys").build()?;
            Ok(())
        })
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(db)
}

pub async fn save_key(key_type: KeyType, data: Vec<u8>) -> Result<(), JsValue> {
    let db = open_db().await?;

    let tx = db
        .transaction("keys")
        .with_mode(TransactionMode::Readwrite)
        .build()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let store = tx
        .object_store("keys")
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let js_key = JsValue::from_str(key_type.as_str());

    let js_val: JsValue = js_sys::Uint8Array::from(data.as_slice()).into();

    store
        .put(js_val.clone())
        .with_key(js_key)
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    console::log_2(
        &JsValue::from_str(&format!(
            "🔑 Saved '{}' key successfully:",
            key_type.as_str()
        )),
        &js_val,
    );

    Ok(())
}
