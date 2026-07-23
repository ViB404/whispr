mod api;
mod crypto;
mod indexeddb;
mod types;

use anyhow::Result;
use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let mut username = use_signal(String::new);

    rsx! {
        h1 { "Generate Keys 🔑" }

        input {
            placeholder: "Username",
            value: "{username}",
            oninput: move |e| username.set(e.value()),
        }

        button {
            onclick: move |_| {
                let username = username();

                spawn_local(async move {
                    if let Err(err) = register(username).await {
                        eprintln!("{err:#}");
                    }
                });
            },

            "Generate"
        }
    }
}

async fn register(username: String) -> Result<()> {
    let keys = crypto::generate_keypair().await?;

    api::register_user::register_user(username, keys.public.clone()).await?;

    indexeddb::save_key(indexeddb::KeyType::Public, keys.public).await?;

    indexeddb::save_key(indexeddb::KeyType::Private, keys.private).await?;

    Ok(())
}
