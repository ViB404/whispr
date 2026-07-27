use crate::api::user::register_user;
use crate::{crypto, indexeddb};
use web_sys::console;

pub async fn register(username: String) -> anyhow::Result<()> {
    console::log_1(&"Generating keypair...".into());

    let keys = crypto::generate_keypair().await?;

    let user = register_user(username, keys.public.clone()).await?;

    indexeddb::save(
        indexeddb::KeyType::Public,
        indexeddb::StoredValue::Bytes(keys.public),
    )
    .await?;

    indexeddb::save(
        indexeddb::KeyType::Private,
        indexeddb::StoredValue::Bytes(keys.private),
    )
    .await?;

    indexeddb::save(
        indexeddb::KeyType::UserId,
        indexeddb::StoredValue::UserId(user.id),
    )
    .await?;

    Ok(())
}
