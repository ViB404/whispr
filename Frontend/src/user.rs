use anyhow::Error;
use wasm_bindgen::JsValue;
use web_sys::console;
use crate::api::user::register_user;
use crate::{crypto, indexeddb};

pub async fn register(username: String) -> Result<(), Error> {
    console::log_1(&"Generating keypair...".into());

    let keys = crypto::generate_keypair().await.expect("There is something went wrong!");

    register_user(username, keys.public.clone()).await.expect("Can't register the user");

    indexeddb::save_key(indexeddb::KeyType::Public, keys.public.clone()).await?;
    indexeddb::save_key(indexeddb::KeyType::Private, keys.private.clone()).await?;

    Ok(())
}